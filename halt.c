/**
 * @file halt.c
 * @brief User-level program to power down the emulator.
 */
#include "types.h"
#include "user.h"

/**
 * @brief Entry point for the halt utility.
 *
 * Prints a message before invoking the `halt` system call. On success the
 * machine will power off and control will not return. The final `exit` call
 * satisfies the usual ABI expectation for returning from main.
 */
int main(void) {
  printf(1, "Shutting down...\n");
  halt();
  exit();
}
