# fc-emulator-rs
 rust fc emulator

# 项目说明

作为模拟器练手，以经典fc红白机为起点，根据 https://www.zhihu.com/column/dustpg 博客逐一复现

语言采用rust，目标是实现一个完整的fc模拟器，包括cpu，ppu，apu，mapper等，最终能够运行fc游戏。

# 测试文件说明

nestest.nes是一个测试文件, 说明文档：http://www.qmtpro.com/~nes/misc/nestest.txt
注：`0x4000 - 0x401F: APU 和 I/O 寄存器` 内部初始状态应设为0xFF


# 进度
- [x] 通过nestest.nes测试文件
  - [x] rom加载与解析
  - [x] cpu指令解析
  - [x] cpu基础、流程指令模拟
  - [x] cpu拓展指令模拟
- [x] 多线程 
- [ ] ppu相关
  - [ ] 背景渲染
    - [ ] 暴力渲染
  - [ ] 精灵渲染
  - [ ] 通过测试文件
- [ ] 输入
- [ ] TAS模拟
- [ ] 超分辨率
- [ ] 多平台
