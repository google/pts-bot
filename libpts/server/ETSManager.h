#ifndef ETSMANAGER_H
#define ETSMANAGER_H

#include <windows.h>
#include <stdbool.h>

typedef bool (__stdcall *LPDEVICESEARCH)(const char *pchAddr, const char *pchName,
                                        const char *pchCod);

typedef bool (__stdcall *LPDONGLEMSG)(const char *pchMsg);

bool InitGetDevInfoWithCallbacks(const char *pchExeInstallDir,
                                 LPDEVICESEARCH devSearchCallback, LPDONGLEMSG dongleMsgCallback);


bool InitEtsEx(const char *pchProfile, const char *pchWorkspacePath,
                const char *pchImplicitSendDllPath, const char *pchPtsDongleAddress);

bool ReinitEtsEx(const char *pchProfile);

typedef bool (__cdecl *LPUSEAUTOIMPLICITSEND)(void);

typedef char *(__cdecl *LPAUTOIMPLICITSEND)(char *description, UINT style);

typedef bool (__stdcall *LPLOG)(const char *pchLogTime, const char *pchLogDescription,
                               const char *pchLogMessage, int nLogType, void *pProject);

typedef bool (__stdcall *LPDEVICESEARCH)(const char *pchAddr, const char *pchName,
                                        const char *pchCod);

typedef bool (__stdcall *LPDONGLEMSG)(const char *pchMsg);

bool RegisterProfileWithCallbacks(const char *pchProfileName,
                                  LPUSEAUTOIMPLICITSEND useAutoImplicitSendCallback,
                                  LPAUTOIMPLICITSEND autoImplicitSendCallback, LPLOG logCallback,
                                  LPDEVICESEARCH devSearchCallback, LPDONGLEMSG dongleMsgCallback);

bool InitStackEx(const char *pchProfileName);

bool VerifyDongleEx(void);

ULONGLONG GetDongleBDAddress(void);

bool StartDeviceSearchEx(const char *pchFilter, const char *pchMask,
                         const char *pchProfileName);

bool StopDeviceSearchEx(const char *pchProfileName);

void GetDongleDeviceInformation(void);

char *GetDeviceList(void);

void SetPTSDevice(const char *pchDeviceName);

bool SetParameterEx(const char *pchParameterName, const char *pchParameterType,
                    const char *pchParameterValue, const char *pchProfileName);

void SetPostLoggingEx(bool bPostLogging, const char *pchProfileName);

bool StartTestCaseEx(const char *pchTestCaseName, const char *pchProfileName,
                     bool bEnableMaxLog);

bool StopTestCaseEx(const char *pchTestCaseName, const char *pchProfileName);

bool TestCaseFinishedEx(const char *pchTestCaseName, const char *pchProfileName);

bool ExitStackEx(const char *pchProfileName);

bool UnregisterProfileEx(const char *pchProfileName);

bool UnRegisterGetDevInfoEx(void);

/* Bluetooth Protocol Viewer */

bool SnifferInitializeEx(void);

int SnifferRegisterNotificationEx(void);

int SnifferClearEx(void);

bool SnifferIsRunningEx(void);

bool SnifferCanSaveEx(void);

bool SnifferCanSaveAndClearEx(void);

int SnifferSaveEx(const char *pchSavePath);

int SnifferSaveAndClearEx(const char *pchSavePath);

int SnifferLogVerdictDescriptionEx(const char *pchLogString, int nVerdictType,
                                   DWORD msSinceTestCaseStart);

bool SnifferIsConnectedEx(void);

bool SnifferCanClearEx(void);

int SnifferTerminateEx(void);

void InitSniffer(void);

#endif /* ETSMANAGER_H */
