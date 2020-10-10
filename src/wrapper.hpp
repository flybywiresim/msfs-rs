// Windows.h support
#ifndef DWORD
#define WINAPI
#define MAX_PATH 4096
#define FALSE false
#define CALLBACK __stdcall
typedef short WCHAR;
typedef unsigned int BOOL;
typedef unsigned char BYTE;
typedef unsigned short WORD;
typedef unsigned long DWORD;
typedef long HRESULT;
typedef const char* LPCSTR;
typedef void* HANDLE;
typedef HANDLE HWND;
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
