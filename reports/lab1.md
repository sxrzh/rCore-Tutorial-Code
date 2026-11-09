# Chapter3 练习报告  

## 编程作业：实现 `sys_trace`  

`0` 和 `1` 操作直接通过 `unsafe` 将地址转换为指针，直接操作内存；  
`2` 操作实现：在 `TaskManagerInner` 中增加二维数组 `syscall_cnt` 字段，在 `TaskManager` 中提供 `record_syscall` 和 `get_syscall_count` 接口；在每次 `syscall` 调用时先通过 `TASK_MANAGER.record_syscall` 更新系统调用次数；`sys_trace` 中通过 `TASK_MANAGER.get_syscall_count` 获取次数统计。  

## 简答作业

### 1. 出错行为

（使用 RustSBI version 0.3.0-alpha.2, adapting to RISC-V SBI v1.0.0）  

+ `ch2b_bad_address.rs`:  

> [kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.

+ `ch2b_bad_instructions.rs`:  

> [kernel] IllegalInstruction in application, kernel killed it.

+ `ch2b_bad_register.rs`:  

> [kernel] IllegalInstruction in application, kernel killed it.

### 2. trap.S 中两个函数的作用  

1. L40: 刚进入 `__resotre` 时，`sp` 代表 kernel stack 栈顶指针。

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

> 无

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

> 无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。