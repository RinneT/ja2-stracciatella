#pragma once

#include "RustInterface.h"

void LogMessage(bool isAssert, LogLevel level, const char *file, const char *format, ...);

/** Print debug message macro. */
#define SLOGD(FORMAT, ...) LogMessage(false, LogLevel::Debug, __FILE__, FORMAT, ##__VA_ARGS__)

/** Print info message macro. */
#define SLOGI(FORMAT, ...) LogMessage(false, LogLevel::Info,  __FILE__, FORMAT, ##__VA_ARGS__)

/** Print warning message macro. */
#define SLOGW(FORMAT, ...) LogMessage(false, LogLevel::Warn, __FILE__, FORMAT, ##__VA_ARGS__)

/** Print error message macro. */
#define SLOGE(FORMAT, ...) LogMessage(false, LogLevel::Error, __FILE__, FORMAT, ##__VA_ARGS__)

/** Print error message macro. */
#define SLOGA(FORMAT, ...) LogMessage(true, LogLevel::Error, __FILE__, FORMAT, ##__VA_ARGS__)
