// FIXME: Use str.split_once when gLinux rustc version >= 1.52.0
pub fn split_once<'a, 'b>(s: &'a str, separator: &'b str) -> Option<(&'a str, &'a str)> {
    let start = s.find(separator)?;
    let end = start + separator.len();
    Some((&s[..start], &s[end..]))
}
