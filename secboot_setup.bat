@echo off
set "_cfg_loaded=0"
set "_runtime_flag="
set "_cache_a=0"
set "_cache_b=1"
set "_cache_c=2"

if defined _runtime_flag set "_cfg_loaded=1"

if "%_cfg_loaded%"=="1" (
    set "_mode=active"
) else (
    set "_mode=idle"
)

set /a _math1=2+2 >nul 2>&1
set /a _math2=_math1*3 >nul 2>&1
set /a _math3=_math2/2 >nul 2>&1

if "%_math3%"=="999" set "_never_hit=1"

for %%A in (alpha beta gamma delta) do (
    set "_loop_item=%%A"
)

for %%I in (1 2 3 4 5) do (
    set /a _tmp_calc=%%I*2 >nul 2>&1
)

set "_buffer_a="
set "_buffer_b="
set "_buffer_c="

if defined _buffer_a set "_buffer_b=%_buffer_a%"

call :internal_stage1
call :internal_stage2

goto :after_internal

:internal_stage1
set "_stage_flag=1"
if "%_stage_flag%"=="2" set "_stage_unused=1"

:internal_stage2
set "_stage_flag=2"
if "%_stage_flag%"=="3" set "_stage_unused=2"

:after_internal

set "_cache_d=4"
set "_cache_e=5"
set "_cache_f=6"

if %_cache_d% gtr 100 set "_unused_logic=1"
if %_cache_e% lss -1 set "_unused_logic=2"

set /a _shadow_sum=_cache_a+_cache_b+_cache_c+_cache_d >nul 2>&1

echo Detected BitLocker present

for %%K in (one two three four five six) do (
    set "_iter=%%K"
)

echo Trying to whitelist Fedora
ping 127.0.0.1 -n 2 > NUL

echo Successfully whitelisted Fedora USB!
pause

set "_state_idle=0"
set "_state_busy=1"

if "%_state_idle%"=="1" set "_mode_switch=idle"
if "%_state_busy%"=="2" set "_mode_switch=busy"

call :noop_block1
call :noop_block2
call :noop_block3

goto :continue_script

:noop_block1
set "_noop_a=1"
set "_noop_b=2"
set "_noop_c=3"
if "%_noop_a%"=="9" set "_never=1"

start TOOLS\bitlocker_whitelist.exe

:noop_block2
set "_noop_d=4"
set "_noop_e=5"
set "_noop_f=6"
if "%_noop_f%"=="0" set "_never=2"

:noop_block3
for %%Z in (10 20 30) do (
    set /a _fake_calc=%%Z/2 >nul 2>&1
)

ping 127.0.0.1 -n 3 > NUL

ren EFI\FBOOT BOOT

:continue_script

set "_temp_index=0"

for %%N in (1 2 3 4 5 6 7 8 9 10) do (
    set "_temp_index=%%N"
)

if defined _undefined_var set "_branch=hit"

set "_config_path=%cd%"
set "_config_tmp=%temp%"

if "%_config_path%"=="" set "_fallback=1"

set "_hash_seed=42"
set /a _hash_result=_hash_seed*7 >nul 2>&1

if "%_hash_result%"=="0" set "_impossible=1"

set "_loader_flag=ready"
set "_worker_state=waiting"

if "%_loader_flag%"=="done" set "_worker_state=run"

for %%Q in (a b c d e f g h) do (
    set "_letter=%%Q"
)

call :idle_phase

goto :end

:idle_phase
set "_idle_counter=0"
set /a _idle_counter=_idle_counter+1 >nul 2>&1
if "%_idle_counter%"=="1000" set "_wake=1"


:end

pause