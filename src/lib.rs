struct Pattern<T, U> {
    filter: Box<dyn Fn(&T) -> bool>,
    action: Box<dyn Fn(T) -> U>,
}

pub struct Match<I, T, U> {
    iter: I,
    patterns: Vec<Pattern<T, U>>,
}

impl<I, T, U> Match<I, T, U> {
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

    pub fn default<F>(self, default: F) -> MatchDefault<I, T, U, F>
    where
        F: Fn() -> U,
    {
        MatchDefault {
            r#match: self,
            default,
        }
    }
}

impl<I, T, U> Iterator for Match<I, T, U>
where
    I: Iterator<Item = T>,
{
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.find_map(|item| {
            self.patterns
                .iter()
                .find(|pattern| (pattern.filter)(&item))
                .map(|pattern| (pattern.action)(item))
        })
    }
}

pub struct MatchDefault<I, T, U, F> {
    r#match: Match<I, T, U>,
    default: F,
}

impl<I, T, U, F> MatchDefault<I, T, U, F>
where
    F: Fn() -> U,
{
    pub fn arm<G, H>(self, filter: G, action: H) -> Self
    where
        G: Fn(&T) -> bool + 'static,
        H: Fn(T) -> U + 'static,
    {
        self.r#match.arm(filter, action).default(self.default)
    }
}

impl<I, T, U, F> Iterator for MatchDefault<I, T, U, F>
where
    I: Iterator<Item = T>,
    F: Fn() -> U,
{
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.r#match.iter.next()?;
        self.r#match
            .patterns
            .iter()
            .find(|pattern| (pattern.filter)(&item))
            .map(|pattern| (pattern.action)(item))
            .or_else(|| Some((self.default)()))
    }
}

pub trait MatchExt: Iterator + Sized {
    fn match_on<U>(self) -> Match<Self, Self::Item, U>;
}

impl<I: Iterator> MatchExt for I {
    fn match_on<U>(self) -> Match<Self, Self::Item, U> {
        Match {
            iter: self,
            patterns: Vec::new(),
        }
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
