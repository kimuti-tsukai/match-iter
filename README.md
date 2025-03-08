```rs
let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
let result: Vec<_> = data
    .into_iter()
    .match_on()
    .arm(|x| x % 2 == 0, |x| x * 2)
    .arm(|x| x % 2 != 0, |x| x * 3)
    .collect();
assert_eq!(result, vec![3, 4, 9, 8, 15, 12, 21, 16, 27, 20]);
```
