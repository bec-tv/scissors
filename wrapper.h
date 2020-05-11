#include "obs-studio/libobs/obs.h"

#ifdef __cplusplus
extern "C" {
#endif

__declspec(dllexport) void scissors_vec2_set(struct vec2 *dst, float x, float y);

__declspec(dllexport) void scissors_run_qt();

#ifdef __cplusplus
}
#endif
