// Compat defines
#define _MSFS_WASM 1
#define SIMCONNECTAPI __attribute__((visibility("default"))) extern "C" HRESULT
#define FSAPI __attribute__((visibility("default")))

// External API headers, assumes MSFS_SDK in include path
#include <WASM/include/MSFS/Legacy/gauges.h>
#include <WASM/include/MSFS/MSFS.h>
#include <WASM/include/MSFS/MSFS_Render.h>
#include <SimConnect SDK/include/SimConnect.h>
