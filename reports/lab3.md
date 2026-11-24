# Chapter3 练习报告

## 编程作业

### 迁移 `sys_get_time`, `sys_mmap`, `sys_munmap`

注意 `memory_set` 信息从 `TASK_MANAGER` 移动到了 `PROCESSOR` 中。

### 实现 `sys_spawn`

`spawn` 先翻译 `path` 地址并获取 ELF（路径不存在则返回错误）。之后先用 `TaskControlBlock::new` 从 ELF 创建一个新进程并将 `parent` 指针指向父进程，父进程的 `children` 增加一项，之后 `add_task` 加入进程队列。不需要像 `fork` 一样复制父进程的内存空间。

注意不能通过直接调用 `sys_fork` 和 `sys_exec` 实现，因为 `sys_fork` 创建的子进程的 `ra` 寄存器指向 `trap_return` 而不是 `sys_spawn` 内部，因此子进程中 `sys_fork` 会直接返回到 `trap_return` 之后返回到用户态，不会执行返回值为 $0$ 分支中的 `exec`。  

### 实现 stride 调度算法和 `sys_set_priority`

在 `TaskControlBlockInner` 中增加 `stride` 和 `stride_pass` 字段，用优先队列（`BinaryHeap` 实现或 `VecDeque` 暴力扫实现）作为线程队列，每次弹出当前 `stride` 最小的进程并给其 `stride` 加上 `stride_pass`，启动进程。设置 `priority` 时修改 `stride_pass`。

某一次测试结果：

```plain
Usertests: Running ch5_stride
priority = 7, exitcode = 24325200, ratio = 3475028
priority = 6, exitcode = 20814000, ratio = 3469000
priority = 10, exitcode = 35060000, ratio = 3506000
priority = 5, exitcode = 17435600, ratio = 3487120
priority = 9, exitcode = 31794000, ratio = 3532666
priority = 8, exitcode = 27971200, ratio = 3496400
```

多次测试，$\max \frac{\text{runtimes}}{\text{prio}} / \min \frac{\text{runtimes}}{\text{prio}}$ 稳定在 $1.05\pm 0.05$。

**简单证明：** 对于两个进程 $A, B$，初始 stride 值相同，pass 分别为 $\text{pass}_a, \text{pass}_b$，每给这两个进程分配一共 $\text{pass}_a+\text{pass}_b$ 次时间，其中一定有 $\text{pass}_b$ 次分配给 A，$\text{pass}_a$ 次分配给 B，且之后 stride 值仍然相同。因此次数足够大时 $\frac{T_A}{T_B}\approx\frac{\text{pass}_b}{\text{pass}_a}=\frac{\text{prio}_a}{\text{prio}_b}$，对任意两个进程都成立。

## 问答作业

### stride 算法深入  

例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。

+ 实际情况是轮到 p1 执行吗？为什么？

> 不是，因为 `p2.stride += 10` 后 $260$ 超过 `u8` 范围，溢出成 $4$，小于 `p1.stride`，下一次轮到 p2 执行。  

我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明， 在不考虑溢出的情况下 , 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。

+ 为什么？尝试简单说明（不要求严格证明）。

> 反证法，如果 $\text{StrideMax} – \text{StrideMin} = S_a - S_b > \frac{\text{BigStride}}{2}$，则 $\text{StrideMax}$ 对应进程 A 在上一次执行前的 stride 为 $S_a-\text{pass}_a \ge S_a- \frac{\text{BigStride}}{2} > \text{StrideMin}$，但是根据算法此时应该先执行 $\text{StrideMin}$ 对应进程 B，矛盾！因此有 $\text{StrideMax} – \text{StrideMin} \le \frac{\text{BigStride}}{2}$。

+ 已知以上结论，考虑溢出的情况下，可以为 Stride 设计特别的比较器，让 BinaryHeap<Stride> 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 partial_cmp 函数，假设两个 Stride 永远不会相等。

```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        const THRES: u64 = (BIG_STRIDE as u64 + 1) / 2;
        Some(if other.0 < self.0 {
            if self.0 - other.0 <= THRES { Ordering::Less }
            else {  Ordering::Greater } 
        }
        else if other.0 > self.0 {
            if other.0 - self.0 <= THRES { Ordering::Greater }
            else { Ordering::Less }
        }
        else {
            Ordering::Equal
        })
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```

在实际应用这种方法时需要注意 `new` 和 `fork` 创建新进程时新进程 stride 初值不能是 $0$（会违背 `STRIDE_MAX – STRIDE_MIN <= BigStride / 2` 条件导致以上条件不是偏序），可以设为当前所有进程（包括不在队列中的父进程）的 stride 最小值。

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

> 无

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

> 无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。