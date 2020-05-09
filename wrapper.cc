#include "wrapper.h"

extern "C" {
  void scissors_vec2_set(struct vec2 *dst, float x, float y) {
    vec2_set(dst, x, y);
  }
}