#[macro_export]
macro_rules! expect_token {
	($tokens:expr, $i:expr, $pattern:pat) => {
		if let $pattern = $tokens[$i].token {
			$i += 1;
		} else {
			panic!(
				"expected {}, got {:?}",
				stringify!($pattern),
				&$tokens[$i].token
			);
		}
	};
}
