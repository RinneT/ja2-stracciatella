#ifndef FILEMAN_H
#define FILEMAN_H

#include <string>
#include <vector>

#include "sgp/SGPFile.h"
#include "sgp/Types.h"

#ifdef _WIN32
#	define WIN32_LEAN_AND_MEAN
#	include <windows.h>
#else
#	include <glob.h>
#endif

/* Delete the file at path. Returns true iff deleting the file succeeded or
 * the file did not exist in the first place. */
void FileDelete(char const* path);
void FileDelete(const std::string &path);

void FileRead( SGPFile*, void*       pDest, size_t uiBytesToRead);
void FileWrite(SGPFile*, void const* pDest, size_t uiBytesToWrite);
SDL_RWops* FileGetRWOps(SGPFile* const f);

template<typename T, typename U> static inline void FileWriteArray(SGPFile* const f, T const& n, U const* const data)
{
	FileWrite(f, &n, sizeof(n));
	if (n != 0) FileWrite(f, data, sizeof(*data) * n);
}

void  FileSeek(SGPFile*, INT32 distance, FileSeekMode);
INT32 FileGetPos(const SGPFile*);

UINT32 FileGetSize(const SGPFile*);

/* Removes ALL FILES in the specified directory, but leaves the directory alone.
 * Does not affect any subdirectories! */
void EraseDirectory(char const* pcDirectory);

enum FileAttributes
{
	FILE_ATTR_NONE      = 0,
	FILE_ATTR_READONLY  = 1U << 0,
	FILE_ATTR_DIRECTORY = 1U << 1,
	FILE_ATTR_ERROR     = 0xFFFFFFFFU
};
ENUM_BITSET(FileAttributes)

FileAttributes FileGetAttributes(const char* filename);

/* Pass in the Fileman file handle of an OPEN file and it will return..
 * - if its a Real File, the return will be the handle of the REAL file
 * - if its a LIBRARY file, the return will be null */
File* GetRealFileHandleFromFileManFileHandle(const SGPFile* hFile);

//Gets the amount of free space on the hard drive that the main executeablt is runnning from
uint64_t GetFreeSpaceOnHardDriveWhereGameIsRunningFrom(void);

/***
 * New file manager.
 *
 * This class provides clean interface for file operations. */
class FileMan
{
public:

	/** Find config folder and switch into it. */
	static std::string switchTmpFolder(std::string homeDir);

	/** Open file for writing.
	 * If file is missing it will be created.
	 * If file exists, it's content will be removed. */
	static SGPFile* openForWriting(const char *filename, bool truncate=true);

	/** Open file for appending data.
	 * If file doesn't exist, it will be created. */
	static SGPFile* openForAppend(const char *filename);

	/** Open file for reading and writing.
	 * If file doesn't exist, it will be created. */
	static SGPFile* openForReadWrite(const char *filename);

	/** Open file for reading. */
	static SGPFile* openForReading(const char *filename);

	/** Open file for reading. */
	static SGPFile* openForReading(const std::string &filename);

	/** Read the whole file as text. */
	static std::string fileReadText(SGPFile*);

#if CASE_SENSITIVE_FS
	/** Find an object (file or subdirectory) in the given directory in case-independent manner.
	 * @return true when found, return the found name using foundName. */
	static bool findObjectCaseInsensitive(const char *directory, const char *name, bool lookForFiles, bool lookForSubdirs, std::string &foundName);
#endif

	/** Open file in the 'Data' directory in case-insensitive manner. */
	static RustPointer<File> openForReadingCaseInsensitive(const std::string& folderPath, const char* filename);

	/* ------------------------------------------------------------
	 * Other operations
	 * ------------------------------------------------------------ */

	/** Create directory.
	 * If directory already exists, do nothing.
	 * If failed to create, raise an exception. */
	static void createDir(char const* path);

	/** Join two path components. */
	static std::string joinPaths(const char *first, const char *second);

	/** Join two path components. */
	static std::string joinPaths(const std::string &first, const char *second);

	/** Join two path components. */
	static std::string joinPaths(const std::string &first, const std::string &second);

	/** Replace extension of a file. */
	static std::string replaceExtension(const std::string &path, const char *newExtensionWithDot);

	/** Get parent path (e.g. directory path from the full path). */
	static std::string getParentPath(const std::string &path, bool absolute);

	/** Get filename from the path. */
	static std::string getFileName(const std::string &path);

	/** Get filename from the path without extension. */
	static std::string getFileNameWithoutExt(const char *path);
	static std::string getFileNameWithoutExt(const std::string &path);

	static RustPointer<File> openFileForReading(const char* filename);

	/** Open file in the given folder in case-insensitive manner.
	 * @return file descriptor or null if file is not found. */
	static RustPointer<File> openFileCaseInsensitive(const std::string& folderPath, const char* filename, uint8_t open_options);

	/** Convert File to HWFile. */
	static SGPFile* getSGPFileFromFile(File* f);

	/** Replace all \ with / */
	static void slashifyPath(std::string &path);

	/** Check file existance. */
	static bool checkFileExistance(const char *folder, const char *fileName);

	/** Move a file */
	static void moveFile(const char *from, const char *to);

private:
	/** Private constructor to avoid instantiation. */
	FileMan() {};
};

/**
 * Find all files with the given extension in the given directory.
 * @param dirPath Path to the directory
 * @param extension Extension (e.g. "txt")
 * @param caseIncensitive When True, do case-insensitive search even of case-sensitive file-systems.
 * @param returnOnlyNames When True, return only names (without the directory path)
 * @param sortResults When True, sort found paths.
 * @return List of paths (dir + filename). */
std::vector<std::string>
FindFilesInDir(const std::string &dirPath,
		const std::string &ext,
		bool caseIncensitive,
		bool returnOnlyNames,
		bool sortResults = false);

/**
 * Find all files in a directory.
 * @param dirPath Path to the directory
 * @param sortResults When True, sort found paths.
 * @return List of paths (dir + filename). */
std::vector<std::string>
FindAllFilesInDir(const std::string &dirPath, bool sortResults = false);

#endif
