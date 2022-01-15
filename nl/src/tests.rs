use super::*;

fn get_default_args() -> NlArgs {
    NlArgs::from_matches(&cli::create_app().get_matches_from(vec!["nl"]))
}

#[test]
fn verify_default_args() {
    let args = get_default_args();

    assert_eq!(args.body_numbering, Style::Nonempty);
    assert_eq!(args.section_delimiter, String::from("\\:"));
    assert_eq!(args.footer_numbering, Style::None);
    assert_eq!(args.header_numbering, Style::None);
    assert_eq!(args.line_increment, 1);
    assert_eq!(args.join_blank_lines, 1);
    assert_eq!(args.number_format, Format::Rn);
    assert!(!args.no_renumber);
    assert_eq!(args.number_separator, String::from("\t"));
    assert_eq!(args.starting_line_number, 1);
    assert_eq!(args.number_width, 6);
}

mod body_numbering {
    use super::*;

    #[test]
    fn nonempty() {
        let args = get_default_args();
        let mut nl = Nl::new(args);

        assert_eq!(nl.convert_line(String::from("line 1")), "     1\tline 1");
        assert_eq!(nl.convert_line(String::from("line 2")), "     2\tline 2");
        assert_eq!(nl.convert_line(String::from("")), "");
        assert_eq!(nl.convert_line(String::from("line 3")), "     3\tline 3");
    }

    #[test]
    fn all() {
        let mut args = get_default_args();
        args.body_numbering = Style::All;

        let mut nl = Nl::new(args);

        assert_eq!(nl.convert_line(String::from("line 1")), "     1\tline 1");
        assert_eq!(nl.convert_line(String::from("line 2")), "     2\tline 2");
        assert_eq!(nl.convert_line(String::from("")), "     3\t");
        assert_eq!(nl.convert_line(String::from("line 3")), "     4\tline 3");
    }

    #[test]
    fn none() {
        let mut args = get_default_args();
        args.body_numbering = Style::None;

        let mut nl = Nl::new(args);

        assert_eq!(nl.convert_line(String::from("line 1")), "       line 1");
        assert_eq!(nl.convert_line(String::from("line 2")), "       line 2");
        assert_eq!(nl.convert_line(String::from("")), "");
    }

    #[test]
    fn regex() {
        let mut args = get_default_args();
        let reg = Regex::new("^line \\d$").expect("Invalid Regex");
        args.body_numbering = Style::Regex(reg);

        let mut nl = Nl::new(args);

        assert_eq!(nl.convert_line(String::from("line 1")), "     1\tline 1");
        assert_eq!(nl.convert_line(String::from("line 2")), "     2\tline 2");
        assert_eq!(nl.convert_line(String::from("line 22")), "       line 22");
        assert_eq!(nl.convert_line(String::from("")), "");
        assert_eq!(nl.convert_line(String::from("line 5")), "     3\tline 5");
    }
}

#[test]
fn starting_line_number() {
    let mut args = get_default_args();
    args.starting_line_number = 5;
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), "     5\tline 1");
    assert_eq!(nl.convert_line(String::from("line 2")), "     6\tline 2");

    let mut args = get_default_args();
    args.starting_line_number = -5;
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), "    -5\tline 1");
    assert_eq!(nl.convert_line(String::from("line 2")), "    -4\tline 2");
}

#[test]
fn line_increment() {
    let mut args = get_default_args();
    args.line_increment = 5;
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), "     1\tline 1");
    assert_eq!(nl.convert_line(String::from("line 2")), "     6\tline 2");
}

#[test]
fn number_separator() {
    let mut args = get_default_args();
    args.number_separator = String::from("   ");
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), "     1   line 1");
    assert_eq!(nl.convert_line(String::from("line 2")), "     2   line 2");
}

#[test]
fn number_width() {
    let mut args = get_default_args();
    args.number_width = 2;
    args.line_increment = 3;
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), " 1\tline 1");
    assert_eq!(nl.convert_line(String::from("line 2")), " 4\tline 2");
    assert_eq!(nl.convert_line(String::from("line 2")), " 7\tline 2");
    assert_eq!(nl.convert_line(String::from("line 2")), "10\tline 2");

    let mut args = get_default_args();
    args.number_width = 1;
    args.line_increment = 3;
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), "1\tline 1");
    assert_eq!(nl.convert_line(String::from("line 2")), "4\tline 2");
    assert_eq!(nl.convert_line(String::from("line 2")), "7\tline 2");
    assert_eq!(nl.convert_line(String::from("line 2")), "10\tline 2");
}

#[test]
fn join_blank_lines() {
    let mut args = get_default_args();
    args.join_blank_lines = 2;
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), "     1\tline 1");
    nl.convert_line(String::from(""));
    nl.convert_line(String::from(""));
    nl.convert_line(String::from(""));
    nl.convert_line(String::from(""));
    assert_eq!(nl.convert_line(String::from("line 6")), "     2\tline 6");

    let mut args = get_default_args();
    args.join_blank_lines = 2;
    args.body_numbering = Style::All;
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), "     1\tline 1");
    nl.convert_line(String::from(""));
    nl.convert_line(String::from(""));
    nl.convert_line(String::from(""));
    nl.convert_line(String::from(""));
    assert_eq!(nl.convert_line(String::from("line 6")), "     4\tline 6");

    let mut args = get_default_args();
    args.join_blank_lines = 3;
    args.body_numbering = Style::All;
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), "     1\tline 1");
    nl.convert_line(String::from(""));
    nl.convert_line(String::from(""));
    nl.convert_line(String::from(""));
    nl.convert_line(String::from(""));
    assert_eq!(nl.convert_line(String::from("line 6")), "     3\tline 6");

    let mut args = get_default_args();
    args.join_blank_lines = 4;
    args.body_numbering = Style::All;
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), "     1\tline 1");
    nl.convert_line(String::from(""));
    nl.convert_line(String::from(""));
    nl.convert_line(String::from(""));
    assert_eq!(nl.convert_line(String::from("")), "     2\t");
    assert_eq!(nl.convert_line(String::from("line 6")), "     3\tline 6");
}

#[test]
fn section_delimiter() {
    let args = get_default_args();
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), "     1\tline 1");
    assert_eq!(nl.convert_line(String::from("line 2")), "     2\tline 2");
    assert_eq!(nl.convert_line(String::from("\\:\\:\\:")), "");
    assert_eq!(nl.convert_line(String::from("line 3")), "       line 3");
    assert_eq!(nl.convert_line(String::from("\\:\\:")), "");
    assert_eq!(nl.convert_line(String::from("line 4")), "     1\tline 4");
}

#[test]
fn footer_numbering() {
    let args = get_default_args();
    let mut nl = Nl::new(args);

    assert_eq!(nl.section, Section::Body);
    assert_eq!(nl.convert_line(String::from("line 1")), "     1\tline 1");
    assert_eq!(nl.convert_line(String::from("line 2")), "     2\tline 2");
    assert_eq!(nl.convert_line(String::from("\\:")), "");
    assert_eq!(nl.convert_line(String::from("line 3")), "       line 3");
    assert_eq!(nl.section, Section::Footer);

    let mut args = get_default_args();
    args.footer_numbering = Style::Nonempty;
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), "     1\tline 1");
    assert_eq!(nl.convert_line(String::from("line 2")), "     2\tline 2");
    assert_eq!(nl.convert_line(String::from("\\:")), "");
    assert_eq!(nl.convert_line(String::from("line 3")), "     1\tline 3");
    assert_eq!(nl.convert_line(String::from("")), "");

    let mut args = get_default_args();
    args.footer_numbering = Style::All;
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), "     1\tline 1");
    assert_eq!(nl.convert_line(String::from("line 2")), "     2\tline 2");
    assert_eq!(nl.convert_line(String::from("\\:")), "");
    assert_eq!(nl.convert_line(String::from("line 3")), "     1\tline 3");
    assert_eq!(nl.convert_line(String::from("line 4")), "     2\tline 4");
}

#[test]
fn header_numbering() {
    let args = get_default_args();
    let mut nl = Nl::new(args);

    assert_eq!(nl.section, Section::Body);
    assert_eq!(nl.convert_line(String::from("line 1")), "     1\tline 1");
    assert_eq!(nl.convert_line(String::from("line 2")), "     2\tline 2");
    assert_eq!(nl.convert_line(String::from("\\:\\:\\:")), "");
    assert_eq!(nl.convert_line(String::from("line 3")), "       line 3");
    assert_eq!(nl.section, Section::Header);

    let mut args = get_default_args();
    args.header_numbering = Style::Nonempty;
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), "     1\tline 1");
    assert_eq!(nl.convert_line(String::from("line 2")), "     2\tline 2");
    assert_eq!(nl.convert_line(String::from("\\:\\:\\:")), "");
    assert_eq!(nl.convert_line(String::from("line 3")), "     1\tline 3");
    assert_eq!(nl.convert_line(String::from("")), "");

    let mut args = get_default_args();
    args.header_numbering = Style::All;
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), "     1\tline 1");
    assert_eq!(nl.convert_line(String::from("line 2")), "     2\tline 2");
    assert_eq!(nl.convert_line(String::from("\\:\\:\\:")), "");
    assert_eq!(nl.convert_line(String::from("line 3")), "     1\tline 3");
}

#[test]
fn number_format() {
    let mut args = get_default_args();
    args.number_width = 2;
    args.line_increment = 3;
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), " 1\tline 1");
    assert_eq!(nl.convert_line(String::from("line 2")), " 4\tline 2");
    assert_eq!(nl.convert_line(String::from("line 2")), " 7\tline 2");
    assert_eq!(nl.convert_line(String::from("line 2")), "10\tline 2");

    let mut args = get_default_args();
    args.number_width = 2;
    args.line_increment = 3;
    args.number_format = Format::Ln;
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), "1 \tline 1");
    assert_eq!(nl.convert_line(String::from("line 2")), "4 \tline 2");
    assert_eq!(nl.convert_line(String::from("line 2")), "7 \tline 2");
    assert_eq!(nl.convert_line(String::from("line 2")), "10\tline 2");

    let mut args = get_default_args();
    args.number_width = 2;
    args.line_increment = 3;
    args.number_format = Format::Rz;
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), "01\tline 1");
    assert_eq!(nl.convert_line(String::from("line 2")), "04\tline 2");
    assert_eq!(nl.convert_line(String::from("line 2")), "07\tline 2");
    assert_eq!(nl.convert_line(String::from("line 2")), "10\tline 2");
}

#[test]
fn no_renumber() {
    let mut args = get_default_args();
    args.header_numbering = Style::All;
    args.no_renumber = true;
    let mut nl = Nl::new(args);

    assert_eq!(nl.convert_line(String::from("line 1")), "     1\tline 1");
    assert_eq!(nl.convert_line(String::from("line 2")), "     2\tline 2");
    assert_eq!(nl.convert_line(String::from("\\:\\:\\:")), "");
    assert_eq!(nl.convert_line(String::from("line 3")), "     3\tline 3");
}
