use crate::memo_gfk::ReservationFilter;
use atomic_float::AtomicF64;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::sync::{atomic::Ordering::Relaxed, Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Reservation {
    r: Arc<AtomicF64>,
    max_idx: f64,
}
impl Default for Reservation {
    fn default() -> Self {
        Self {
            r: Arc::new(AtomicF64::new(std::f64::MAX)),
            max_idx: std::f64::MAX,
        }
    }
}
impl Reservation {
    fn get(&self) -> f64 {
        self.r.load(Relaxed)
    }

    pub fn reserve(&mut self, i: f64) -> bool {
        if self.r.load(Relaxed) < i as f64 {
            self.r.swap(i, Relaxed);
            true
        } else {
            false
        }
    }

    pub fn reserved(&self) -> bool {
        return self.r.load(Relaxed) < self.max_idx;
    }

    pub fn reset(&mut self) {
        self.r.swap(self.max_idx, Relaxed);
    }

    pub fn freeze(&mut self) {
        self.r.swap(-1.0, Relaxed);
    }

    pub fn check(&self, i: f64) -> bool {
        return self.r.load(Relaxed) == i;
    }

    pub fn check_reset(&mut self, i: f64) -> bool {
        if self.r.load(Relaxed) == i {
            self.r.swap(self.max_idx, Relaxed);
            return true;
        } else {
            return false;
        }
    }
}

pub fn speculative_for<T>(
    step: &Arc<Mutex<T>>,
    s: f64,
    e: f64,
    granularity: f64,
    has_state: bool,
    max_tries: f64,
) -> f64
where
    T: ReservationFilter + std::marker::Send + std::marker::Sync,
{
    if max_tries < 0.0 {
        let mut max_tries = 100. + 200. * granularity;
    } else {
        ()
    }

    let max_round_size: f64 = f64::max(4., (e - s) / granularity + 1.);
    let mut current_round_size: f64 = max_round_size / 4.;

    let mut i_hold: Vec<f64> = Vec::new();
    let mut state: Arc<Mutex<Vec<T>>> = Arc::new(Mutex::new(Vec::new()));
    let mut i: Arc<Mutex<Vec<f64>>> =
        Arc::new(Mutex::new(Vec::with_capacity(max_round_size as usize)));
    let mut keep: Arc<Mutex<Vec<bool>>> =
        Arc::new(Mutex::new(Vec::with_capacity(max_round_size as usize)));

    let mut round: f64 = 0.;
    let mut number_done: f64 = s;
    let mut number_keep = 0;
    let mut total_processed: f64 = 0.;

    while number_done < e {
        round += 1.;
        if round > max_tries {
            panic!("Speculative_for: too many iterations, increase MaxTries");
        }

        let size: f64 = f64::min(current_round_size, e - number_done);

        total_processed += size;
        let loop_granularity: usize = 0;

        if has_state {
            (0..s as usize).into_par_iter().for_each(|j| {
                i.lock().unwrap()[j] = if j < number_keep {
                    i_hold[j]
                } else {
                    number_done + 1.
                };
                keep.lock().unwrap()[j] = state.lock().unwrap()[j].reserve(i.lock().unwrap()[j]);
            });
        } else {
            (0..s as usize).into_par_iter().for_each(|j| {
                i.lock().unwrap()[j] = if j < number_keep {
                    i_hold[j]
                } else {
                    number_done + 1.
                };
                keep.lock().unwrap()[j] = step.lock().unwrap().reserve(i.lock().unwrap()[j]);
            });
        }

        if has_state {
            (0..size as usize).into_par_iter().for_each(|j| {
                if keep.lock().unwrap()[j] {
                    keep.lock().unwrap()[j] =
                        !state.lock().unwrap()[j].commit(i.lock().unwrap()[j]);
                }
            });
        } else {
            (0..size as usize).into_par_iter().for_each(|j| {
                if keep.lock().unwrap()[j] {
                    keep.lock().unwrap()[j] = !step.lock().unwrap().commit(i.lock().unwrap()[j]);
                }
            });
        }
        let i_hold = &i.lock().unwrap()[0..size as usize]
            .iter()
            .zip(&keep.lock().unwrap()[0..size as usize])
            .filter(|(_, i)| **i)
            .map(|(j, _)| *j)
            .collect::<Vec<f64>>();

        let number_keep: usize = i_hold.len();
        number_done += size - number_keep as f64;
        if (number_keep as f64 / size) > 0.2 {
            current_round_size = f64::max(
                current_round_size / 2.,
                f64::max(max_round_size / 64.0 + 1., number_keep as f64),
            );
        } else if (number_keep as f64 / size) < 0.1 {
            current_round_size = f64::min(current_round_size * 2., max_round_size);
        }
    }
    return total_processed;
}
