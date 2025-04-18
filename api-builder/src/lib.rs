mod spec;

#[cfg(test)]
mod test {
    #[test]
    fn open_api() {
        let api = super::spec::Specification::new();
        let json = api.json().unwrap();
        let json = serde_json::to_string_pretty(&json).unwrap();
        println!("{json}");
        assert!(false);
    }
}
