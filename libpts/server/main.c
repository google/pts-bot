#include <windows.h>
#include <winreg.h>
#include <stdio.h>
#include <pthread.h>

#include "ETSManager.h"


pthread_mutex_t stdout_mutex = PTHREAD_MUTEX_INITIALIZER;
pthread_mutex_t test_mutex = PTHREAD_MUTEX_INITIALIZER;

static void json_escape(const char *string) {
	for (size_t i = 0; string[i] != '\0'; i++) {
		switch (string[i]) {
		case '"': printf("\\\""); break;
		case '\\': printf("\\\\"); break;
		case '\b': printf("\\b"); break;
		case '\f': printf("\\f"); break;
		case '\n': printf("\\n"); break;
		case '\r': printf("\\r"); break;
		case '\t': printf("\\t"); break;
		default:
			if ('\x00' <= string[i] && string[i] <= '\x1f') {
				printf("\\u%04x", string[i]);
			} else {
				printf("%c", string[i]);
			}
		}
	}
}

static bool __stdcall on_device(const char* pchAddr, const char* pchName, const
                            char* pchCod) {
	(void)pchAddr;
	(void)pchName;
	(void)pchCod;
	return true;
}

static bool __stdcall on_dongle_msg(const char *message) {
	(void)message;
	return true;
}

static bool __cdecl on_use_auto_implicit_send(void) {
	return true;
}

static char * __cdecl on_implicit_send(char *description, UINT style) {
	pthread_mutex_lock(&stdout_mutex);
	printf("{\"type\": \"implicit_send\", \"description\": \"");
	json_escape(description);
	printf("\", \"style\": %u}\n", style);
	pthread_mutex_unlock(&stdout_mutex);

	char *answer = NULL;
	size_t n = 0;

	if (getline(&answer, &n, stdin) == -1) {
		fprintf(stderr, "getline failed\n");
		exit(1);
	}

	return answer;
}

#define LOG_TYPE_FINAL_VERDICT (5)

static bool __stdcall on_log(const char *time, const char *description, const char *message, int type, void *user) {
	(void)user;

	pthread_mutex_lock(&stdout_mutex);
	printf("{\"type\": \"log\", \"time\": \"");
	json_escape(time);
	printf("\", \"description\": \"");
	json_escape(description);
	printf("\", \"message\": \"");
	json_escape(message);
	printf("\", \"logtype\": %d}\n", type);
	pthread_mutex_unlock(&stdout_mutex);

	// Test ended
	if (type == LOG_TYPE_FINAL_VERDICT && strstr(message, "VERDICT/") != NULL) {
		pthread_mutex_unlock(&test_mutex);
	}

	return true;
}

#define PORTS_CLASS_GUID "{4D36E978-E325-11CE-BFC1-08002BE10318}"

static LSTATUS register_port(const char *port) {
	char name[256];

	snprintf(name, sizeof(name), "System\\CurrentControlSet\\Enum\\VIRTUAL\\VID_1915&PID_521F\\%s", port);

	HKEY key;
	LSTATUS status;

	status = RegCreateKeyExA(
		HKEY_LOCAL_MACHINE,
		name,
		0,
		NULL,
		REG_OPTION_NON_VOLATILE,
		KEY_ALL_ACCESS,
		NULL,
		&key,
		NULL
	);
	if (status != ERROR_SUCCESS) return status;

	status = RegSetValueExA(key, "ClassGUID", 0, REG_SZ, (const BYTE *)PORTS_CLASS_GUID, sizeof(PORTS_CLASS_GUID));
	if (status != ERROR_SUCCESS) goto ret;

	int len = snprintf(name, sizeof(name), "HCI (%s)", port);

	status = RegSetValueExA(key, "FriendlyName", 0, REG_SZ, (const BYTE *)name, len + 1);
	if (status != ERROR_SUCCESS) goto ret;

ret:
	RegCloseKey(key);
	return status;
}

int main(int argc, char *argv[]) {
	if (argc < 3) {
		fprintf(stderr, "Usage: %s [port] [profile]\n", argv[0]);
		return 1;
	}

	char const *port = argv[1];
	char const *profile = argv[2];
	char const *test = argv[3];

	char directory[256];
	GetModuleFileName(NULL, directory, sizeof(directory));
	*strrchr(directory, '\\') = '\0';

	bool success;

	success = RegisterProfileWithCallbacks(
		profile,
		on_use_auto_implicit_send,
		on_implicit_send,
		on_log,
		on_device,
		on_dongle_msg
	);
	if (!success) return 1;

	success = InitGetDevInfoWithCallbacks(directory, on_device, on_dongle_msg);
	if (!success) return 1;

	LSTATUS status = register_port(port);
	if (status != ERROR_SUCCESS){
		fprintf(stderr, "Failed to register port\n");
		return 1;
	}
	char const *devices = GetDeviceList();

	if (strstr(devices, port) == NULL) {
		fprintf(stderr, "Failed to register device\n");
		return 1;
	}

	SetPTSDevice(port);

	success = VerifyDongleEx();
	//if (!success) return 1;

	char addr[13];
	snprintf(addr, sizeof(addr), "%012llX",GetDongleBDAddress());

	printf("{\"type\": \"addr\", \"value\": \"%s\"}\n", addr);

	GetDongleDeviceInformation();

	for (int i = 4; i < argc; i += 3) {
		success = SetParameterEx(argv[i], argv[i + 1], argv[i + 2], profile);

		if (!success) {
			fprintf(stderr, "SetParameterEx failed %s %s %s\n", argv[i], argv[i + 1], argv[i + 2]);
			return 1;
		}
	}

	char implicit_send[sizeof(directory) + 20];
	snprintf(implicit_send, sizeof(implicit_send), "%s\\implicit_send3.dll", directory);

	success = InitEtsEx(profile, "NIL", implicit_send, addr);

	if (!success) {
		fprintf(stderr, "InitEtsEx failed\n");
		return 1;
	}

	success = InitStackEx(profile);
	if (!success) {
		fprintf(stderr, "InitStackEx failed\n");
		return 1;
	}

	SetPostLoggingEx(false, profile);

	pthread_mutex_lock(&test_mutex);

	StartTestCaseEx(test, profile, true);

	pthread_mutex_lock(&test_mutex);

	TestCaseFinishedEx(test, profile);
	ExitStackEx(profile);
	UnregisterProfileEx(profile);
	UnRegisterGetDevInfoEx();
}
