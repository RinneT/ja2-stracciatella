#pragma once

#include "sgp/FileMan.h"

/** Get location of directory with extra data. */
std::string GetExtraDataDir();

/** Open test resource file for reading. */
SGPFile* OpenTestResourceForReading(const char *filePath);
