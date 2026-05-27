/*
 * Minimal shim. sha2.c includes both postgres.h and postgres_fe.h
 * depending on compile context. We define the same typedefs in both
 * via this include-the-other-shim trick.
 */
#ifndef PG_SHIM_POSTGRES_FE_H
#define PG_SHIM_POSTGRES_FE_H
#include "postgres.h"
#endif
