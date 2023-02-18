use crate::memo_gfk::ReservationFilter;
use atomic_float::AtomicF64;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::sync::{
    atomic::{AtomicI64, Ordering::Relaxed},
    Arc, Mutex,
};

#[derive(Debug, Clone)]
pub struct Reservation {
    r: Arc<AtomicI64>,
    max_idx: i64,
}
impl Default for Reservation {
    fn default() -> Self {
        Self {
            r: Arc::new(AtomicI64::new(std::i64::MAX)),
            max_idx: std::i64::MAX,
        }
    }
}
impl Reservation {
    fn get(&self) -> i64 {
        self.r.load(Relaxed)
    }

    pub fn reserve(&mut self, i: i64) -> bool {
        let temp = i64::min(self.r.load(Relaxed), i);
        if self.r.load(Relaxed) > i {
            self.r.swap(temp, Relaxed);
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
        self.r.swap(-1, Relaxed);
    }

    pub fn check(&self, i: i64) -> bool {
        return self.r.load(Relaxed) == i;
    }

    pub fn check_reset(&mut self, i: i64) -> bool {
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
    s: i64,
    e: i64,
    granularity: i64,
    has_state: bool,
    max_tries: i64,
) -> i64
where
    T: ReservationFilter + std::marker::Send + std::marker::Sync,
{
    let mut max_tries = if max_tries < 0 {
        100 + 200 * granularity
    } else {
        max_tries
    };

    let max_round_size: i64 = i64::max(4, ((e - s) / granularity) + 1);
    let mut current_round_size: i64 = max_round_size / 4;

    let mut i_hold: Vec<i64> = Vec::new();
    let mut state = Arc::new(Mutex::new(Vec::<T>::new()));

    let mut i: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(vec![0; max_round_size as usize]));
    let mut keep: Arc<Mutex<Vec<bool>>> =
        Arc::new(Mutex::new(vec![false; max_round_size as usize]));

    let mut round: i64 = 0;
    let mut number_done: i64 = s;
    let mut number_keep = 0;
    let mut total_processed: i64 = 0;
    while number_done < e {
        round += 1;
        if round > max_tries {
            panic!("Speculative_for: too many iterations, increase MaxTries");
        }

        let size: i64 = i64::min(current_round_size, e - number_done);
        total_processed += size;
        let loop_granularity: usize = 0;
        if has_state {
            (0..size as usize).into_par_iter().for_each(|j| {
                i.lock().unwrap()[j] = if j < number_keep {
                    i_hold[j]
                } else {
                    number_done + j as i64
                };
                keep.lock().unwrap()[j] = state.lock().unwrap()[j].reserve(i.lock().unwrap()[j]);
            });
        } else {
            (0..size as usize).into_par_iter().for_each(|j| {
                if j < number_keep {
                    i.lock().unwrap()[j] = i_hold[j]
                } else {
                    i.lock().unwrap()[j] = number_done + j as i64
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
        i_hold = i.lock().unwrap()[0..size as usize]
            .to_vec()
            .iter()
            .zip(&keep.lock().unwrap()[0..size as usize].to_vec())
            .filter(|(_, i)| **i)
            .map(|(j, _)| *j)
            .collect::<Vec<i64>>();

        number_keep = i_hold.len();
        number_done += size - number_keep as i64;
        if (number_keep as i64 / size) as f32 > 0.2 {
            current_round_size = i64::max(
                current_round_size / 2,
                i64::max((max_round_size / 64) + 1, number_keep as i64),
            );
        } else if ((number_keep as i64 / size) as f32) < 0.1 {
            current_round_size = i64::min(current_round_size * 2, max_round_size);
        }
    }
    return total_processed;
}

#[cfg(test)]
mod tests {
    use crate::speculative_for::Reservation;

    #[test]
    pub fn speculative_test() {
        let mut test = Reservation::default();
        println!("Reservation {:?}", test);
        println!(" Reservation Get {:?}", test.get());
        println!(" Reservation Reserved {:?}", test.reserved());
        println!(" Reservation Reserved {:?}", test.freeze());
        println!("Reservation {:?}", test.check(-1));
        println!("Reservation {:?}", test.check_reset(-1));
        println!(" Reservation Reserved {:?}", test.freeze());
        println!("Reservation {:?}", test.reset());
        println!("Reservation {:?}", test);
        println!(" Reservation Reserve {:?}", test.reserve(100));
        println!(" Reservation Reserve {:?}", test.reserve(110));
        println!(" Reservation Reserve {:?}", test.reserve(10));
        println!(" Reservation Reserve {:?}", test);
        println!(" Reservation Reserve {:?}", test.reserve(100));
    }
}
