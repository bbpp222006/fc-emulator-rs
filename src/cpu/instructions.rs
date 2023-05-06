//包含一个枚举类型，用于表示所有可能的 6502 指令，以及与指令解码和执行相关的逻辑。例如，可以实现一个从操作码到指令的映射函数（from_opcode），以及一个执行指令的方法（execute）。
// src/cpu/instructions.rs

use crate::cpu::addressing_modes::AddressingMode;

/// 6502 CPU 的指令集枚举
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Instruction {
    LDA, // 累加器寻址
    LDX, // X 寄存器寻址
    LDY, // Y 寄存器寻址
    BRK, // 中断
    SEI, // 禁用中断
    NOP, // 空指令
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

/// 操作码和指令的映射关系
#[derive(Clone)]
pub struct Opcode {
    pub code: u8,
    pub instruction: Instruction,
    pub addressing_mode: AddressingMode,
}

macro_rules! opcode {
    ($($code:expr => $instr:ident, $mode:ident),+) => {
        {
            use Instruction::*;
            use AddressingMode::*;

            let mut opcode_map: Vec<Opcode> = Vec::with_capacity(256);
            opcode_map.resize(256, Opcode { code: 0, instruction: NOP, addressing_mode: Implied });

            $(
                opcode_map[$code as usize] = Opcode { code: $code, instruction: $instr, addressing_mode: $mode };
            )+

            opcode_map
        }
    };
}

impl Opcode {
    /// 根据操作码返回对应的指令和寻址模式
    pub fn decode(opcode: u8) -> (Instruction, AddressingMode) {
        // 创建一个预定义的操作码到指令和寻址模式的映射
        let opcode_map: Vec<Opcode> = opcode![
        0x00 => BRK, Implied,0x01 => ORA, IndirectX,0x02 => STP, Implied,0x03 => SLO, IndirectX,0x04 => NOP, ZeroPage,0x05 => ORA, ZeroPage,0x06 => ASL, ZeroPage,0x07 => SLO, ZeroPage,0x08 => PHP, Implied,0x09 => ORA, Immediate,0x0A => ASL, Accumulator,0x0B => ANC, Immediate,0x0C => NOP, Absolute,0x0D => ORA, Absolute,0x0E => ASL, Absolute,0x0F => SLO, Absolute,
        0x10 => BPL, Relative,0x11 => ORA, IndirectY,0x12 => STP, Implied,0x13 => SLO, IndirectY,0x14 => NOP, ZeroPageX,0x15 => ORA, ZeroPageX,0x16 => ASL, ZeroPageX,0x17 => SLO, ZeroPageX,0x18 => CLC, Implied,0x19 => ORA, AbsoluteY,0x1A => NOP, Implied,0x1B => SLO, AbsoluteY,0x1C => NOP, AbsoluteX,0x1D => ORA, AbsoluteX,0x1E => ASL, AbsoluteX,0x1F => SLO, AbsoluteX,
        0x20 => JSR, Absolute,0x21 => AND, IndirectX,0x22 => STP, Implied,0x23 => RLA, IndirectX,0x24 => BIT, ZeroPage,0x25 => AND, ZeroPage,0x26 => ROL, ZeroPage,0x27 => RLA, ZeroPage,0x28 => PLP, Implied,0x29 => AND, Immediate,0x2A => ROL, Accumulator,0x2B => ANC, Immediate,0x2C => BIT, Absolute,0x2D => AND, Absolute,0x2E => ROL, Absolute,0x2F => RLA, Absolute,
        0x30 => BMI, Relative,0x31 => AND, IndirectY,0x32 => STP, Implied,0x33 => RLA, IndirectY,0x34 => NOP, ZeroPageX,0x35 => AND, ZeroPageX,0x36 => ROL, ZeroPageX,0x37 => RLA, ZeroPageX,0x38 => SEC, Implied,0x39 => AND, AbsoluteY,0x3A => NOP, Implied,0x3B => RLA, AbsoluteY,0x3C => NOP, AbsoluteX,0x3D => AND, AbsoluteX,0x3E => ROL, AbsoluteX,0x3F => RLA, AbsoluteX,
        0x40 => RTI, Implied,0x41 => EOR, IndirectX,0x42 => STP, Implied,0x43 => SRE, IndirectX,0x44 => NOP, ZeroPage,0x45 => EOR, ZeroPage,0x46 => LSR, ZeroPage,0x47 => SRE, ZeroPage,0x48 => PHA, Implied,0x49 => EOR, Immediate,0x4A => LSR, Accumulator,0x4B => ALR, Immediate,0x4C => JMP, Absolute,0x4D => EOR, Absolute,0x4E => LSR, Absolute,0x4F => SRE, Absolute,
        0x50 => BVC, Relative,0x51 => EOR, IndirectY,0x52 => STP, Implied,0x53 => SRE, IndirectY,0x54 => NOP, ZeroPageX,0x55 => EOR, ZeroPageX,0x56 => LSR, ZeroPageX,0x57 => SRE, ZeroPageX,0x58 => CLI, Implied,0x59 => EOR, AbsoluteY,0x5A => NOP, Implied,0x5B => SRE, AbsoluteY,0x5C => NOP, AbsoluteX,0x5D => EOR, AbsoluteX,0x5E => LSR, AbsoluteX,0x5F => SRE, AbsoluteX,
        0x60 => RTS, Implied,0x61 => ADC, IndirectX,0x62 => STP, Implied,0x63 => RRA, IndirectX,0x64 => NOP, ZeroPage,0x65 => ADC, ZeroPage,0x66 => ROR, ZeroPage,0x67 => RRA, ZeroPage,0x68 => PLA, Implied,0x69 => ADC, Immediate,0x6A => ROR, Accumulator,0x6B => ARR, Immediate,0x6C => JMP, Indirect,0x6D => ADC, Absolute,0x6E => ROR, Absolute,0x6F => RRA, Absolute,
        0x70 => BVS, Relative,0x71 => ADC, IndirectY,0x72 => STP, Implied,0x73 => RRA, IndirectY,0x74 => NOP, ZeroPageX,0x75 => ADC, ZeroPageX,0x76 => ROR, ZeroPageX,0x77 => RRA, ZeroPageX,0x78 => SEI, Implied,0x79 => ADC, AbsoluteY,0x7A => NOP, Implied,0x7B => RRA, AbsoluteY,0x7C => NOP, AbsoluteX,0x7D => ADC, AbsoluteX,0x7E => ROR, AbsoluteX,0x7F => RRA, AbsoluteX,
        0x80 => NOP, Immediate,0x81 => STA, IndirectX,0x82 => NOP, Immediate,0x83 => SAX, IndirectX,0x84 => STY, ZeroPage,0x85 => STA, ZeroPage,0x86 => STX, ZeroPage,0x87 => SAX, ZeroPage,0x88 => DEY, Implied,0x89 => NOP, Immediate,0x8A => TXA, Implied,0x8B => XAA, Immediate,0x8C => STY, Absolute,0x8D => STA, Absolute,0x8E => STX, Absolute,0x8F => SAX, Absolute,
        0x90 => BCC, Relative,0x91 => STA, IndirectY,0x92 => STP, Implied,0x93 => AHX, IndirectY,0x94 => STY, ZeroPageX,0x95 => STA, ZeroPageX,0x96 => STX, ZeroPageY,0x97 => SAX, ZeroPageY,0x98 => TYA, Implied,0x99 => STA, AbsoluteY,0x9A => TXS, Implied,0x9B => TAS, AbsoluteY,0x9C => SHY, AbsoluteX,0x9D => STA, AbsoluteX,0x9E => SHX, AbsoluteY,0x9F => AHX, AbsoluteY,
        0xA0 => LDY, Immediate,0xA1 => LDA, IndirectX,0xA2 => LDX, Immediate,0xA3 => LAX, IndirectX,0xA4 => LDY, ZeroPage,0xA5 => LDA, ZeroPage,0xA6 => LDX, ZeroPage,0xA7 => LAX, ZeroPage,0xA8 => TAY, Implied,0xA9 => LDA, Immediate,0xAA => TAX, Implied,0xAB => LAX, Immediate,0xAC => LDY, Absolute,0xAD => LDA, Absolute,0xAE => LDX, Absolute,0xAF => LAX, Absolute,
        0xB0 => BCS, Relative,0xB1 => LDA, IndirectY,0xB2 => STP, Implied,0xB3 => LAX, IndirectY,0xB4 => LDY, ZeroPageX,0xB5 => LDA, ZeroPageX,0xB6 => LDX, ZeroPageY,0xB7 => LAX, ZeroPageY,0xB8 => CLV, Implied,0xB9 => LDA, AbsoluteY,0xBA => TSX, Implied,0xBB => LAS, AbsoluteY,0xBC => LDY, AbsoluteX,0xBD => LDA, AbsoluteX,0xBE => LDX, AbsoluteY,0xBF => LAX, AbsoluteY,
        0xC0 => CPY, Immediate,0xC1 => CMP, IndirectX,0xC2 => NOP, Immediate,0xC3 => DCP, IndirectX,0xC4 => CPY, ZeroPage,0xC5 => CMP, ZeroPage,0xC6 => DEC, ZeroPage,0xC7 => DCP, ZeroPage,0xC8 => INY, Implied,0xC9 => CMP, Immediate,0xCA => DEX, Implied,0xCB => AXS, Immediate,0xCC => CPY, Absolute,0xCD => CMP, Absolute,0xCE => DEC, Absolute,0xCF => DCP, Absolute,
        0xD0 => BNE, Relative,0xD1 => CMP, IndirectY,0xD2 => STP, Implied,0xD3 => DCP, IndirectY,0xD4 => NOP, ZeroPageX,0xD5 => CMP, ZeroPageX,0xD6 => DEC, ZeroPageX,0xD7 => DCP, ZeroPageX,0xD8 => CLD, Implied,0xD9 => CMP, AbsoluteY,0xDA => NOP, Implied,0xDB => DCP, AbsoluteY,0xDC => NOP, AbsoluteX,0xDD => CMP, AbsoluteX,0xDE => DEC, AbsoluteX,0xDF => DCP, AbsoluteX,
        0xE0 => CPX, Immediate,0xE1 => SBC, IndirectX,0xE2 => NOP, Immediate,0xE3 => ISC, IndirectX,0xE4 => CPX, ZeroPage,0xE5 => SBC, ZeroPage,0xE6 => INC, ZeroPage,0xE7 => ISC, ZeroPage,0xE8 => INX, Implied,0xE9 => SBC, Immediate,0xEA => NOP, Implied,0xEB => SBC, Immediate,0xEC => CPX, Absolute,0xED => SBC, Absolute,0xEE => INC, Absolute,0xEF => ISC, Absolute,
        0xF0 => BEQ, Relative, 0xF1 => SBC, IndirectY, 0xF2 => STP, Implied, 0xF3 => ISC, IndirectY, 0xF4 => NOP, ZeroPageX, 0xF5 => SBC, ZeroPageX, 0xF6 => INC, ZeroPageX, 0xF7 => ISC, ZeroPageX, 0xF8 => SED, Implied, 0xF9 => SBC, AbsoluteY, 0xFA => NOP, Implied, 0xFB => ISC, AbsoluteY, 0xFC => NOP, AbsoluteX, 0xFD => SBC, AbsoluteX, 0xFE => INC, AbsoluteX, 0xFF => ISC, AbsoluteX
        ];

        // 在映射中查找给定操作码
        if let Some(opcode_struct) = opcode_map.iter().find(|op| op.code == opcode) {
            (opcode_struct.instruction, opcode_struct.addressing_mode)
        } else {
            panic!("无效的操作码: {:02X}", opcode);
        }
    }
}
