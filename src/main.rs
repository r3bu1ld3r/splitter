// Implement basic function to split some generic computational work between threads.
// Split should occur only on some threshold - if computational work (input length) is shorter than this threshold,
// no splitting should occur and no threads should be created.
//
// You get as input:
//
// 1. Vec<T>
// 2. Function f(t: T) -> R
//
//
// Threshold can be just constant.
//
// You should return:
//    1. Up to you, but probably some Vec of the same length as input(1)
//
// Code should be published on github.

use std::{
    sync::{mpsc::channel, Arc, Mutex},
    thread,
};

use rayon::prelude::*;

const TRESHOLD: usize = 100;

pub fn splitter_with_rayon<T, R>(input: Vec<T>, f: fn(T) -> R) -> Vec<R>
where
    T: Clone + Send + Sync,
    R: Send,
{
    if input.len() < TRESHOLD {
        input.iter().map(|v| f(v.clone())).collect()
    } else {
        input
            .par_chunks(TRESHOLD)
            .flat_map(|chunk| chunk.iter().map(|v| f(v.clone())).collect::<Vec<R>>())
            .collect()
    }
}

pub fn splitter_no_deps<T, R>(input: Vec<T>, f: fn(T) -> R) -> Vec<R>
where
    T: Clone + Send + Sync + 'static,
    R: Send + 'static,
{
    if input.len() < TRESHOLD {
        input.iter().map(|v| f(v.clone())).collect()
    } else {
        let mut handles = vec![];
        let mut res: Vec<R> = vec![];
        for chunk in input.chunks(TRESHOLD) {
            let (tx, rx) = channel::<Vec<T>>();
            let syncable = Arc::new(Mutex::new(rx));
            handles.push(thread::spawn(move || {
                syncable
                    .lock()
                    .unwrap()
                    .recv()
                    .unwrap()
                    .iter()
                    .map(|v| f(v.clone()))
                    .collect::<Vec<R>>()
            }));
            tx.send(chunk.to_vec()).unwrap();
        }

        for h in handles {
            let mut v = h.join().unwrap();
            res.append(&mut v);
        }
        res
    }
}

#[test]
pub fn without_splitting() {
    let input = vec![10, 20, 30, 40, 50];

    let modifier = |x: u32| -> String { x.to_string() };

    let result_rayon = splitter_with_rayon(input.clone(), modifier);
    let result_no_deps = splitter_no_deps(input.clone(), modifier);

    let correct = input
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    assert_eq!(result_rayon, correct);
    assert_eq!(result_no_deps, correct);
}

#[test]
pub fn with_splitting() {
    let input = (0..100000).into_iter().collect::<Vec<u32>>();

    let modifier = |x: u32| -> String { x.to_string() };

    let result_rayon = splitter_with_rayon(input.clone(), modifier);
    let result_no_deps = splitter_no_deps(input.clone(), modifier);

    let correct = input
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    assert_eq!(result_rayon, correct);
    assert_eq!(result_no_deps, correct);
}

fn main() {}
