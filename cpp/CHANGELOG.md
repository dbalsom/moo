# MOO ChangeLog
 ## v1.1
  - Refactored `Reader` into class
  - Added optional gzip support (define `MOO_USE_ZLIB`)
  - Added << operator for register enums. 
  - Don't mask initial registers
  - Added enum iterator for `REG16` and `REG32` enums
  - MooHeader.GetVersion now returns std::pair<>
  - Added a basic `CMakeLists.txt` file for building the example program

 ## v1.0 - Initial release