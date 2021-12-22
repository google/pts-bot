use crate::compat::split_once;

pub fn id_to_mmi(profile: &str, id: u32) -> Option<&'static str> {
    include!("../data/mmi_ids.inc.rs")
}

pub fn parse(description: &str) -> Option<(&str, &str, &str, &str)> {
    let description = description.strip_prefix("{")?;
    let (header, description) = split_once(description, "}")?;
    let (id, header) = split_once(header, ",")?;
    let (test, profile) = split_once(header, ",")?;

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
    fn test_id_to_mmi() {
        assert_eq!(
            id_to_mmi("A2DP", 1002),
            Some("TSC_AVDTP_mmi_iut_accept_connect")
        )
    }
}
