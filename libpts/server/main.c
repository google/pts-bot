// Copyright 2025 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#include <windows.h>
#include <winreg.h>
#include <stdio.h>
#include <pthread.h>

#include "ETSManager.h"

pthread_mutex_t stdout_mutex = PTHREAD_MUTEX_INITIALIZER;
pthread_mutex_t test_mutex = PTHREAD_MUTEX_INITIALIZER;

static wchar_t cp1252_to_unicode(char c) {
	switch ((unsigned char)c) {
		case 0x80: return 0x20AC; // EURO SIGN
		case 0x82: return 0x201A; // SINGLE LOW-9 QUOTATION MARK
		case 0x83: return 0x0192; // LATIN SMALL LETTER F WITH HOOK
		case 0x84: return 0x201E; // DOUBLE LOW-9 QUOTATION MARK
		case 0x85: return 0x2026; // HORIZONTAL ELLIPSIS
		case 0x86: return 0x2020; // DAGGER
		case 0x87: return 0x2021; // DOUBLE DAGGER
		case 0x88: return 0x02C6; // MODIFIER LETTER CIRCUMFLEX ACCENT
		case 0x89: return 0x2030; // PER MILLE SIGN
		case 0x8A: return 0x0160; // LATIN CAPITAL LETTER S WITH CARON
		case 0x8B: return 0x2039; // SINGLE LEFT-POINTING ANGLE QUOTATION MARK
		case 0x8C: return 0x0152; // LATIN CAPITAL LIGATURE OE
		case 0x8E: return 0x017D; // LATIN CAPITAL LETTER Z WITH CARON
		case 0x91: return 0x2018; // LEFT SINGLE QUOTATION MARK
		case 0x92: return 0x2019; // RIGHT SINGLE QUOTATION MARK
		case 0x93: return 0x201C; // LEFT DOUBLE QUOTATION MARK
		case 0x94: return 0x201D; // RIGHT DOUBLE QUOTATION MARK
		case 0x95: return 0x2022; // BULLET
		case 0x96: return 0x2013; // EN DASH
		case 0x97: return 0x2014; // EM DASH
		case 0x98: return 0x02DC; // SMALL TILDE
		case 0x99: return 0x2122; // TRADE MARK SIGN
		case 0x9A: return 0x0161; // LATIN SMALL LETTER S WITH CARON
		case 0x9B: return 0x203A; // SINGLE RIGHT-POINTING ANGLE QUOTATION MARK
		case 0x9C: return 0x0153; // LATIN SMALL LIGATURE OE
		case 0x9E: return 0x017E; // LATIN SMALL LETTER Z WITH CARON
		case 0x9F: return 0x0178; // LATIN CAPITAL LETTER Y WITH DIAERESIS
		default: return c;
	}
}

static void json_escape_cp1252(const char *string) {
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
				printf("%lc", cp1252_to_unicode(string[i]));
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

#define MMI_STYLE_OK_CANCEL_2 (0x11141)

static char * __cdecl on_implicit_send(char *description, UINT style) {
	pthread_mutex_lock(&stdout_mutex);
	printf("{\"type\": \"implicit_send\", \"description\": \"");
	json_escape_cp1252(description);
	printf("\", \"style\": %u}\n", style);
	pthread_mutex_unlock(&stdout_mutex);

	char *answer = NULL;
	size_t n = 0;

	static unsigned to_skip = 0;

	/* From Implicit_Send_8.0.3.pdf 3.4 MMI styles
	 *
	 * Note: When ImplicitSendStyle() is called with style MMI_Style_Ok_Cancel2,
	 * implementation may signal the IUT the requested action after the message tag is
	 * identified but it should not block in the function. Otherwise, it may block PTS from progressing.
	 * Implementation should always return “OK”.
	 */
	if (style == MMI_STYLE_OK_CANCEL_2) {
		to_skip++;
		return "OK";
	}

	/* Skip all the answer that we ignored and read one more answer */
	for (unsigned i = 0; i < to_skip + 1; i++) {
		if (getline(&answer, &n, stdin) == -1) {
			fprintf(stderr, "getline failed\n");
			exit(1);
		}
	}
	to_skip = 0;

	return answer;
}

#define LOG_TYPE_FINAL_VERDICT (5)

static bool __stdcall on_log(const char *time, const char *description, const char *message, int type, void *user) {
	(void)user;

	pthread_mutex_lock(&stdout_mutex);
	printf("{\"type\": \"log\", \"time\": \"");
	json_escape_cp1252(time);
	printf("\", \"description\": \"");
	json_escape_cp1252(description);
	printf("\", \"message\": \"");
	json_escape_cp1252(message);
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

	success = InitEtsEx(profile, "C:\\workspace", implicit_send, addr);

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
