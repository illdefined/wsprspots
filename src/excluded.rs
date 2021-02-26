use phf::{Set, phf_set};

/// Set of excluded call signs
pub static EXCLUDED: Set<&'static str> = phf_set! {
	// Not logging WSPR QSOs
	"DF2JP",
	"DL6WAB"
};
