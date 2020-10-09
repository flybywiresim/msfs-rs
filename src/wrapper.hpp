// Windows.h support
#ifndef DWORD
#include <linux/limits.h>
#include <stdint.h>
#define WINAPI
#define MAX_PATH PATH_MAX
#define FALSE false
#define CALLBACK
typedef short WCHAR;
typedef void* HANDLE;
typedef unsigned int BOOL;
typedef unsigned char BYTE;
typedef unsigned short WORD;
typedef unsigned long DWORD;
typedef long HRESULT;
typedef HANDLE HWND;
typedef const char* LPCSTR;
typedef struct _GUID {
  unsigned long Data1;
  unsigned short Data2;
  unsigned short Data3;
  unsigned char Data4[8];
} GUID;
#endif  // ifndef DWORD

// Compat defines
#define _MSFS_WASM 1

// External API headers, assumes MSFS_SDK in include path
#include <WASM/include/MSFS/MSFS.h>
#include <WASM/include/MSFS/MSFS_Render.h>
#include <SimConnect SDK/include/SimConnect.h>
