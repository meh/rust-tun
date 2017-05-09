error_chain! {
	errors {
		NameTooLong
		UnsupportedFamily
	}

	foreign_links {
		Io(::std::io::Error);
	}
}
