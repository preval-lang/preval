cd stdlib
clang-cl print.c /LD /O2 /MD /I . /link Ws2_32.lib Ntdll.lib Kernel32.lib Userenv.lib Advapi32.lib Bcrypt.lib /OUT:print.dll
cd ..
