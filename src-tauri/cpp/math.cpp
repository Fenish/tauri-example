#include <cstdint>

extern "C" {
    int32_t sum(int32_t a, int32_t b) {
        return a + b;
    }

	int32_t subtract(int32_t a, int32_t b) {
		return a - b;
	}

	int32_t multiply(int32_t a, int32_t b) {
		return a * b;
	}

	int32_t divide(int32_t a, int32_t b) {
		return a / b;
	}

	int32_t stress_test(int32_t a){
		int32_t result = 0;
		for(int32_t i = 0; i < a; i++){
			result += i;
		}
		return result;
	}
} 