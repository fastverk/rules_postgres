/* tools/postgres_src/overlay/pg_config_os.h
 *
 * Stand-in for the configure-selected platform header. Configure
 * normally chooses one of src/include/port/<os>.h and writes the
 * selected contents here. We just defer to the right one based on
 * the C compiler's platform macros — works for darwin and linux,
 * which are the only platforms this experiment targets.
 */
#ifndef PG_CONFIG_OS_H
#define PG_CONFIG_OS_H

#if defined(__APPLE__)
#include "port/darwin.h"
#elif defined(__linux__)
#include "port/linux.h"
#else
#error "Unsupported OS for this experimental Bazel build."
#endif

#endif /* PG_CONFIG_OS_H */
