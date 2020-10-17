// Compat defines
#define SIMCONNECTAPI __attribute__((visibility("default"))) extern "C" HRESULT
#define FSAPI __attribute__((visibility("default")))

#include <MSFS/MSFS.h>
#include <MSFS/MSFS_Render.h>
#include <MSFS/MSFS_WindowsTypes.h>
#include <SimConnect.h>
#include <MSFS/Legacy/gauges.h>

// Exports using the FSAPI definition in MSFS/Legacy/gauges.h need to be explicitly exported due to the FSAPI macro
// being redefined without visibility

#define FSAPI __attribute__((visibility("default")))

FLOAT64 FSAPI aircraft_varget(ENUM simvar, ENUM units, SINT32 index);
BOOL FSAPI execute_calculator_code(PCSTRINGZ code, FLOAT64* fvalue, SINT32* ivalue, PCSTRINGZ* svalue);
ENUM FSAPI get_aircraft_var_enum(PCSTRINGZ simvar);
ENUM FSAPI get_units_enum(PCSTRINGZ unitname);
