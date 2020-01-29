//todo: try returning an iterator instead to avoid allocating the vec if caller doesnt need it
pub fn cartesian_product<T: Copy>(lists: &[Vec<T>]) -> Vec<Vec<T>> {
    lists.iter().fold(vec![vec![]], |product, list| {
        list.iter().flat_map(|item| {
            product.iter().map(|prev_tuple| {
                let mut new_tuple = prev_tuple.clone();
                new_tuple.push(*item);
                new_tuple
            }).collect::<Vec<Vec<T>>>()
        }).collect::<Vec<Vec<T>>>()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test() {
        let result = cartesian_product(&[
            vec!["1", "2"],
            vec!["a", "b"],
            vec!["x", "y", "z"],
        ]);

        assert_eq!(result, &[
            ["1", "a", "x"],
            ["2", "a", "x"],
            ["1", "b", "x"],
            ["2", "b", "x"],
            ["1", "a", "y"],
            ["2", "a", "y"],
            ["1", "b", "y"],
            ["2", "b", "y"],
            ["1", "a", "z"],
            ["2", "a", "z"],
            ["1", "b", "z"],
            ["2", "b", "z"],
        ]);
    }
}
