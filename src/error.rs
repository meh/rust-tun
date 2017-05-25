error_chain! {
	errors {
		NameTooLong
		InvalidAddress
	}

	foreign_links {
		Io(::std::io::Error);
		Nul(::std::ffi::NulError);
	}
}
