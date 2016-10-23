If /i "%CONFIGURATION%"=="release" (
    SET cfg=--release
) ELSE (
    SET cfg
)
SET cfgcmd=cargo test --verbose %cfg%
ECHO %cfgcmd%
call %%cfgcmd%%
