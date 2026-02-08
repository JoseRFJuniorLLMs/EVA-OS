#ifndef EVA_NPU_H
#define EVA_NPU_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

int32_t eva_npu_init(void);

void eva_npu_shutdown(void);

void *eva_npu_alloc(uintptr_t size);

void eva_npu_free(void *ptr);

int32_t eva_npu_memcpy_to_device(void *dst, const void *src, uintptr_t size);

int32_t eva_npu_memcpy_from_device(void *dst, const void *src, uintptr_t size);

int32_t eva_npu_execute(const void *blob,
                        uintptr_t blob_size,
                        const void *const *inputs,
                        void **outputs,
                        uintptr_t num_inputs,
                        uintptr_t num_outputs);

uint64_t eva_npu_get_total_memory(void);

uint64_t eva_npu_get_available_memory(void);

const char *eva_npu_get_device_name(void);

#endif  /* EVA_NPU_H */
