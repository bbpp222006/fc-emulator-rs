# fc-emulator-rs
 rust fc emulator

# 项目说明

作为模拟器练手，以经典fc红白机为起点，根据 https://www.zhihu.com/column/dustpg 博客逐一复现

语言采用rust，目标是实现一个完整的fc模拟器，包括cpu，ppu，apu，mapper等，最终能够运行fc游戏。

# 测试文件说明

nestest.nes是一个测试文件, 说明文档：http://www.qmtpro.com/~nes/misc/nestest.txt
注：`0x4000 - 0x401F: APU 和 I/O 寄存器` 内部初始状态应设为0xFF


# 输入说明

w a s d 为方向键，j k 为A B键，回车键为start，空格键为select

# 进度
- [x] 通过nestest.nes测试文件
  - [x] rom加载与解析
  - [x] cpu指令解析
  - [x] cpu基础、流程指令模拟
  - [x] cpu拓展指令模拟
- [ ] 多线程 
  - [x] 暴力实现
  - [ ] 整理管道命名
- [ ] ppu相关
  - [ ] 通过测试文件
  - [ ] 中断交互
  - [ ] 背景渲染
    - [x] 暴力渲染
  - [ ] 精灵渲染
    - [ ] OAM DMA 实现
- [ ] 输入
  - [x] 键盘输入
  - [ ] 手柄输入
  - [ ] 双手柄模拟
  - [x] 单手柄模拟
- [ ] TAS模拟
- [ ] 超分辨率
- [ ] 多平台
- [ ] 性能优化