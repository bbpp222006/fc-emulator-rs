//包含一个枚举类型，用于表示所有可能的 6502 指令，以及与指令解码和执行相关的逻辑。例如，可以实现一个从操作码到指令的映射函数（from_opcode），以及一个执行指令的方法（execute）。

use crate::cpu::AddressingMode;
use crate::cpu::opcodes::decode_opcode;

#[derive(Debug)]
pub enum InstructionType {
    Common,
    CrossingPage,
    Branch,
}

pub struct InstructionInfo {
    pub operand_code: u8,
    pub instruction: Instruction,
    pub addressing_mode: AddressingMode,
    pub operand_size: u8,
    pub instruction_cycle: u8,
    pub instruction_type:InstructionType,
    pub unofficial:bool,
}

impl std::default::Default for InstructionInfo {
    fn default() -> Self {
        decode_opcode(0xEA) //默认是nop
    }
}

/// 6502 CPU 的指令集枚举
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Instruction {
    LDA, // 累加器寻址
    LDX, // X 寄存器寻址
    LDY, // Y 寄存器寻址
    BRK, // 中断
    SEI, // 禁用中断
    NOP, // 空指令
    DOP, // *双重NOP （拓展指令）
    TOP, // *三重NOP （拓展指令）
    PHP, // 将状态寄存器压入堆栈
    BPL, // 分支指令
    CLC, // 清除进位标志
    ORA, // 或操作
    STP, // 停止
    ASL, // 算术左移
    SLO, // ASL + ORA
    ROL, // 循环左移
    ANC, // AND + Carry
    JSR, // 跳转到子程序
    BIT, // 测试位
    PLP, // 从堆栈中弹出状态寄存器
    BMI, // 分支指令
    SEC, // 设置进位标志
    AND, // 与操作
    RLA, // ROL + AND
    RTI, // 从中断返回
    PHA, // 将累加器压入堆栈
    JMP, // 跳转
    BVC, // 分支指令
    CLI, // 清除禁用中断标志
    EOR, // 异或操作
    LSR, // 逻辑右移
    SRE, // LSR + EOR
    RTS, // 从子程序返回
    PLA, // 从堆栈中弹出累加器
    BVS, // 分支指令
    ADC, // 加法
    ROR, // 循环右移
    RRA, // ROR + ADC
    STY, // 存储 Y 寄存器
    DEY, // 减少 Y 寄存器
    BCC, // 分支指令
    TYA, // 将 Y 寄存器复制到累加器
    SHY, // 未实现
    STA, // 存储累加器
    STX, // 存储 X 寄存器
    TXA, // 将 X 寄存器复制到累加器
    TXS, // 将 X 寄存器复制到堆栈指针
    SHX, // 未实现
    SAX, // 未实现
    XAA, // 未实现
    AHX, // 未实现
    TAS, // 未实现
    TAY, // 将累加器复制到 Y 寄存器
    BCS, // 分支指令
    CLV, // 清除溢出标志
    TAX, // 将累加器复制到 X 寄存器
    TSX, // 将堆栈指针复制到 X 寄存器
    LAX, // LDA + LDX
    LAS, // LDA + LDX + S
    CPY, // 比较 Y 寄存器
    INY, // 增加 Y 寄存器
    BNE, // 分支指令
    CLD, // 清除十进制模式标志
    CMP, // 比较累加器
    DEC, // 减少内存值
    DEX, // 减少 X 寄存器
    DCP, // DEC + CMP
    AXS, // 未实现
    CPX, // 比较 X 寄存器
    INX, // 增加 X 寄存器
    BEQ, // 分支指令
    SED, // 设置十进制模式标志
    SBC, // 减法
    INC, // 增加内存值
    ISC, // INC + SBC
    ALR, // AND + LSR
    ARR, // AND + ROR
}



