chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold:
	sub sp, sp, #96
	stp x29, x30, [sp, #32]
	stp x24, x23, [sp, #48]
	stp x22, x21, [sp, #64]
	stp x20, x19, [sp, #80]
	add x29, sp, #32
	mov w8, wzr
	mov w20, #2
	add x21, x0, #296
	mov w9, #1
.LBB9_1:
	mov x14, x3
	b .LBB9_5
	cmp x13, x14
	b.eq .LBB9_11
.LBB9_3:
	bfi x12, x14, #32, #32
	dmb ish
.LBB9_4:
	ldr x13, [x0, #416]
	dmb ishld
	mov x14, x10
	ldr x19, [x13, x11, lsl #3]
	dmb ishld
	add x11, x0, #128
	casal x14, x12, [x11]
	cmp x14, x10
	b.eq .LBB9_15
.LBB9_5:
	ldp x13, x12, [x0, #424]
	mov x10, x14
	and x11, x12, x14
	sub x13, x13, #1
	cmp x11, x13
	b.ne .LBB9_7
	orr w12, w10, w12
	and x13, x10, #0xffffffff00000000
	add w12, w12, #1
	orr x12, x12, x13
	lsr x14, x10, #32
	mov w13, w10
	cmp x13, x14
	b.ne .LBB9_4
	b .LBB9_8
.LBB9_7:
	add x12, x10, #1
	lsr x14, x10, #32
	mov w13, w10
	cmp x13, x14
	b.ne .LBB9_4
.LBB9_8:
	ldr x14, [x0, #128]
	cmp x14, x10
	b.ne .LBB9_5
	add x15, x0, #440
	ldar x16, [x15]
	ldar x14, [x0]
	mov w14, w14
	cbz x16, .LBB9_2
	orr x16, x20, x14, lsl #2
	mov w17, #1
	casal x17, x16, [x15]
	lsr x15, x17, #2
	cmp x17, #1
	csel x14, x14, x15, eq
	cmp x13, x14
	b.ne .LBB9_3
	b .LBB9_17
.LBB9_11:
	tbnz w8, #0, .LBB9_19
	ldr x8, [x2]
	mov x19, x10
	mov x22, x0
	cbnz x8, .LBB9_14
	stp x21, xzr, [x2]
	stp xzr, xzr, [x2, #16]
	strb w20, [x2, #40]
	stp x1, x1, [sp]
	mov x23, x1
	mov x1, sp
	mov x0, x2
	str xzr, [sp, #16]
	mov x24, x2
	bl aiq::wait_queue::Wait<Q,SP>::poll_wait
	mov x1, x23
	mov x2, x24
	mov w8, w0
	mov x0, x22
	mov w9, #1
	mov x3, x19
	b .LBB9_1
.LBB9_15:
	add x8, x0, #288
	ldar x1, [x8]
	cmp x1, #1
	b.ls .LBB9_18
.LBB9_16:
	mov x9, xzr
.LBB9_17:
	mov x0, x9
	mov x1, x19
	ldp x20, x19, [sp, #80]
	ldp x22, x21, [sp, #64]
	ldp x24, x23, [sp, #48]
	ldp x29, x30, [sp, #32]
	add sp, sp, #96
	ret
.LBB9_18:
	add x0, x0, #256
	bl spmc_waker::SpmcWaker<_,_>::wake_unsync_cold
	b .LBB9_16
	mov w9, #2
	b .LBB9_17
