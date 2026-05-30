chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold:
	sub sp, sp, #112
	stp x29, x30, [sp, #32]
	str x25, [sp, #48]
	stp x24, x23, [sp, #64]
	stp x22, x21, [sp, #80]
	stp x20, x19, [sp, #96]
	add x29, sp, #32
	mov w8, wzr
	add x19, x1, #256
	mov w20, #2
.LBB14_1:
	mov x13, x4
	b .LBB14_3
.LBB14_2:
	mov x13, x10
	casal x13, x11, [x1]
	cmp x13, x10
	b.eq .LBB14_14
.LBB14_3:
	ldp x12, x11, [x1, #448]
	mov x10, x13
	and x9, x11, x13
	sub x12, x12, #1
	cmp x9, x12
	cset w12, hi
	csinv w12, w12, wzr, hs
	and w12, w12, #0xff
	b.eq .LBB14_6
	cmp w12, #255
	b.ne .LBB14_15
	add x11, x10, #1
	mov w12, w10
	cmp x12, x10, lsr #32
	b.ne .LBB14_2
	b .LBB14_7
.LBB14_6:
	orr w11, w10, w11
	and x12, x10, #0xffffffff00000000
	add w11, w11, #1
	orr x11, x11, x12
	mov w12, w10
	cmp x12, x10, lsr #32
	b.ne .LBB14_2
.LBB14_7:
	ldar x13, [x1]
	cmp x13, x10
	b.ne .LBB14_3
	add x13, x1, #128
	ldar x13, [x13]
	ldr w14, [x1, #456]
	add w13, w13, w14
	add w13, w13, #1
	cmp x13, x12
	b.eq .LBB14_10
	bfi x11, x13, #32, #32
	b .LBB14_2
.LBB14_10:
	tbnz w8, #0, .LBB14_17
	ldr x8, [x3]
	mov x21, x10
	mov x22, x1
	mov x23, x0
	cbnz x8, .LBB14_13
	stp x19, xzr, [x3]
	stp xzr, xzr, [x3, #16]
	strb w20, [x3, #40]
	mov x1, sp
	mov x0, x3
	stp x2, x2, [sp]
	str xzr, [sp, #16]
	mov x24, x3
	mov x25, x2
	bl aiq::wait_queue::Wait<Q,SP>::poll_wait
	mov x2, x25
	mov x3, x24
	mov w8, w0
	mov x0, x23
	mov x1, x22
	mov x4, x21
	b .LBB14_1
.LBB14_14:
	ldr x8, [x1, #440]
	str xzr, [x0]
	add x8, x8, x9, lsl #4
	stp x8, x12, [x0, #8]
	b .LBB14_16
.LBB14_15:
	stp xzr, xzr, [x0]
.LBB14_16:
	ldp x20, x19, [sp, #96]
	ldr x25, [sp, #48]
	ldp x22, x21, [sp, #80]
	ldp x24, x23, [sp, #64]
	ldp x29, x30, [sp, #32]
	add sp, sp, #112
	ret
	mov w8, #1
	str x8, [x0]
	b .LBB14_16
