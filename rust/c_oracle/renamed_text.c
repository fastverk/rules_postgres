/*
 * renamed_text.c — rename textcat → textcat_orig before compiling,
 * so the test linker doesn't merge with the Rust crate's #[no_mangle].
 *
 * text_catenate is `static text *` — file-local linkage, no rename
 * needed.
 */

#define textcat       textcat_orig

#include "text.c"
