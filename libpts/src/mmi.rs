pub fn id_to_mmi(profile: &str, id: u32) -> Option<&'static str> {
    include!("../data/mmi_ids.inc.rs")
}

pub fn parse(description: &str) -> Option<(&str, &str, &str, &str)> {
    let (header, description) = description.strip_prefix('{')?.split_once('}')?;
    let (id, header) = header.split_once(',')?;
    let (test, profile) = header.split_once(',')?;

    Some((id, test.trim(), profile.trim(), description))
}

#[cfg(test)]
mod test {
    use super::id_to_mmi;
    use super::parse;

    #[test]
    fn test_parse() {
        assert_eq!(
            parse("{1002,A2DP/SNK/AS/BV-01-I,A2DP}If necessary, take action ..."),
            Some((
                "1002",
                "A2DP/SNK/AS/BV-01-I",
                "A2DP",
                "If necessary, take action ..."
            )),
        );
    }
    #[test]
    fn test_parse_without_description() {
        assert_eq!(
            parse("{test_started,foo,bar}"),
            Some(("test_started", "foo", "bar", "")),
        );
    }

    #[test]
    fn test_id_to_mmi() {
        assert_eq!(
            id_to_mmi("A2DP", 1002),
            Some("TSC_AVDTP_mmi_iut_accept_connect")
        )
    }
}
