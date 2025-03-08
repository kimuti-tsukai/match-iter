struct Pattern<T, U> {
    filter: Box<dyn Fn(&T) -> bool>,
    action: Box<dyn Fn(T) -> U>,
}

pub struct Match<I, T, U, D = fn() -> U> {
    iter: I,
    patterns: Vec<Pattern<T, U>>,
    default: Option<D>,
}

impl<I, T, U, D> Match<I, T, U, D> {
    fn new(iter: I) -> Self {
        Self {
            iter,
            patterns: Vec::new(),
            default: None,
        }
    }

    pub fn arm<F, G>(mut self, filter: F, action: G) -> Self
    where
        F: Fn(&T) -> bool + 'static,
        G: Fn(T) -> U + 'static,
    {
        self.patterns.push(Pattern {
            filter: Box::new(filter),
            action: Box::new(action),
        });
        self
    }

    pub fn default(self, default: D) -> Self {
        Self {
            default: Some(default),
            ..self
        }
    }
}

impl<I, T, U, D> Iterator for Match<I, T, U, D>
where
    I: Iterator<Item = T>,
    D: Fn() -> U,
{
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.find_map(|item| {
            self.patterns
                .iter()
                .find(|pattern| (pattern.filter)(&item))
                .map(|pattern| (pattern.action)(item))
                .or_else(|| self.default.as_ref().map(|f| f()))
        })
    }
}

pub trait MatchExt: Iterator + Sized {
    fn match_on<U>(self) -> Match<Self, Self::Item, U>;
}

impl<I: Iterator> MatchExt for I {
    fn match_on<U>(self) -> Match<Self, Self::Item, U> {
        Match::new(self)
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use crate::MatchExt;

    #[test]
    fn test_match() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result: Vec<_> = data
            .into_iter()
            .match_on()
            .arm(|x| x % 2 == 0, |x| x * 2)
            .arm(|x| x % 2 != 0, |x| x * 3)
            .collect();
        assert_eq!(result, vec![3, 4, 9, 8, 15, 12, 21, 16, 27, 20]);
    }

    #[test]
    fn partial_match() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result: Vec<_> = data
            .into_iter()
            .match_on()
            .arm(|x| x % 2 == 0, |x| x * 2)
            .collect();
        assert_eq!(result, vec![4, 8, 12, 16, 20]);
    }

    #[test]
    fn default_test() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result: Vec<_> = data
            .into_iter()
            .match_on()
            .arm(|x| x % 2 == 0, |x| x * 2)
            .default(|| 0)
            .collect();
        assert_eq!(result, vec![0, 4, 0, 8, 0, 12, 0, 16, 0, 20]);
    }

    #[test]
    fn once_test() {
        let data = iter::once(1);
        let result = data
            .match_on()
            .arm(|x| x % 2 == 0, |x| x * 2)
            .arm(|x| x % 3 == 0, |x| x * 3)
            .default(|| 0)
            .next()
            .unwrap();
        assert_eq!(result, 0);
    }
}
