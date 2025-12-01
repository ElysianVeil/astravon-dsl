; ModuleID = 'module'
source_filename = "module"

@str_0 = private unnamed_addr constant [16 x i8] c"Hello Astravon!\00", align 1

define i32 @main() {
entry:
  %call_puts = call i32 @puts(i8* getelementptr inbounds ([16 x i8], [16 x i8]* @str_0, i32 0, i32 0))
  ret i32 0
}

declare i32 @puts(i8*)
