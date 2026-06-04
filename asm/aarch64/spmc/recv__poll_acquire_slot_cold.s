chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold:
	sub sp, sp, #96
	stp x29, x30, [sp, #32]
	stp x24, x23, [sp, #48]
	stp x22, x21, [sp, #64]
	stp x20, x19, [sp, #80]
	add x29, sp, #32
	mov x8, x0
	mov w0, wzr
	mov w19, #2
	add x20, x8, #296
	mov w9, #1
.LBB9_1:
	mov x14, x3
	b .LBB9_5
	mov w14, w15
	cmp x10, x14
	b.eq .LBB9_11
.LBB9_3:
	bfi x13, x14, #32, #32
.LBB9_4:
	ldr x10, [x8, #416]
	mov x14, x11
	ldr x10, [x10, x12, lsl #3]
	dmb ishld
	add x12, x8, #128
	casal x14, x13, [x12]
	cmp x14, x11
	b.eq .LBB9_15
.LBB9_5:
	ldp x13, x10, [x8, #424]
	mov x11, x14
	and x12, x10, x14
	sub x13, x13, #1
	cmp x12, x13
	b.ne .LBB9_7
	orr w10, w11, w10
	and x13, x11, #0xffffffff00000000
	add w10, w10, #1
	orr x13, x10, x13
	lsr x14, x11, #32
	mov w10, w11
	cmp x10, x14
	b.ne .LBB9_4
	b .LBB9_8
.LBB9_7:
	add x13, x11, #1
	lsr x14, x11, #32
	mov w10, w11
	cmp x10, x14
	b.ne .LBB9_4
.LBB9_8:
	add x14, x8, #128
	ldar x14, [x14]
	cmp x14, x11
	b.ne .LBB9_5
	add x14, x8, #440
	ldar x15, [x8]
	ldar x16, [x14]
	cbz x16, .LBB9_2
	ldar x15, [x8]
	mov w16, #2
	mov w17, #1
	bfi x16, x15, #2, #32
	mov w15, w15
	casal x17, x16, [x14]
	lsr x14, x17, #2
	cmp x17, #1
	csel x14, x15, x14, eq
	cmp x10, x14
	b.ne .LBB9_3
	b .LBB9_16
.LBB9_11:
	tbnz w0, #0, .LBB9_17
	mov x22, x8
	ldr x8, [x2]
	mov x21, x11
	cbnz x8, .LBB9_14
	stp x20, xzr, [x2]
	stp xzr, xzr, [x2, #16]
	strb w19, [x2, #40]
	stp x1, x1, [sp]
	mov x23, x1
	mov x1, sp
	mov x0, x2
	str xzr, [sp, #16]
	mov x24, x2
	bl aiq::wait_queue::Wait<Q,SP>::poll_wait
	mov x1, x23
	mov x2, x24
	mov x8, x22
	mov w9, #1
	mov x3, x21
	b .LBB9_1
.LBB9_15:
	mov x9, xzr
.LBB9_16:
	mov x0, x9
	mov x1, x10
	ldp x20, x19, [sp, #80]
	ldp x22, x21, [sp, #64]
	ldp x24, x23, [sp, #48]
	ldp x29, x30, [sp, #32]
	add sp, sp, #96
	ret
	mov w9, #2
	b .LBB9_16
