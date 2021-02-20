use phf::{Set, phf_set};

/// Set of excluded call signs
pub static EXCLUDED: Set<&'static str> = phf_set! {
	"DL6WAB"  // Not logging WSPR QSOs
};
