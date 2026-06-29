#ifndef POWER_CORE_H
#define POWER_CORE_H

char* get_script_ultra_perf(int cores, const char* install_cmd);
char* get_script_perf(const char* install_cmd);
char* get_script_balanced(const char* install_cmd);
char* get_script_save(int cores, const char* install_cmd);
char* get_script_ultra_save(int cores, const char* install_cmd);

#endif