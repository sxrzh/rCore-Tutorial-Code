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

### 2. trap.S 的理解

1. L40: 刚进入 `__resotre` 时，`sp` 代表保存了 Trap 上下文后的内核栈栈顶指针。两种使用情景：  
    1. 启动一个 app 运行第一个任务时（`run_first_task`），通过 `TaskContext::goto_restore` 中设置的 `ra` 寄存器返回到 `__restore`，初始化寄存器并从 S 态进入 U 态；
    2. 每次 trap 结束时，通过 `__restore` 从 Trap 上下文恢复 `__alltraps` 保存的寄存器值，并从 S 态返回 U 态。

2. L43-L48：特殊处理了 `sstatus`, `sepc`, `sscratch` 寄存器，它们都是需要 S 模式才能访问的 CSR 寄存器。`sstatus` 指定从 S 模式返回后的 S 状态（其中 SPP 位记录进入 trap 前的特权级），指定返回后为 U 特权级模式；`sepc` 指定 trap 的返回地址；`sscratch` 存储进入 trap 前的用户栈顶指针，之后恢复到 `sp` 寄存器。  

3. L50-L56：`x2` 就是 `sp`，保存在 `2*8(sp)` 和 `sscratch`，`csrrw sp, sscratch, sp` 就是恢复 `x2`；`x4` 是线程指针寄存器 `tp`，目前单线程系统中不使用。  

4. L60：该指令后 `sp` 是用户栈栈顶，`sscratch` 是释放 trap 上下文后的内核栈栈顶。  

5. `__restore` 中发生状态切换在 `sret` 指令。该指令执行后由于 `sstatus` 寄存器中的 SPP 位为 0（User）故进入用户态。

6. L13：该指令后 `sp` 是分配 trap 上下文前的内核栈栈顶，`sscratch` 是用户栈栈顶。

7. 从 U 态进入 S 态是在用户库的 `syscall.rs` 中调用 `ecall` 指令时。

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

> 无

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

> 无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。