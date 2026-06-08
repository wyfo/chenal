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
.LBB9_1:
	ldr x9, [x1, #432]
	mov x14, x4
	b .LBB9_3
.LBB9_2:
	mov x14, x11
	cas x14, x12, [x1]
	cmp x14, x11
	b.eq .LBB9_16
.LBB9_3:
	ldr x12, [x1, #424]
	and x10, x9, x14
	mov x11, x14
	sub x12, x12, #1
	cmp x10, x12
	cset w12, hi
	csinv w12, w12, wzr, hs
	ands w12, w12, #0xff
	b.eq .LBB9_6
	cmp w12, #255
	b.ne .LBB9_14
	add x12, x11, #1
	mov w13, w11
	cmp x13, x11, lsr #32
	b.eq .LBB9_7
	b .LBB9_2
.LBB9_6:
	orr w12, w11, w9
	and x13, x11, #0xffffffff00000000
	add w12, w12, #1
	orr x12, x12, x13
	mov w13, w11
	cmp x13, x11, lsr #32
	b.ne .LBB9_2
.LBB9_7:
	ldr x14, [x1]
	cmp x14, x11
	b.ne .LBB9_3
	add x9, x1, #128
	ldar x14, [x9]
	ldr x9, [x1, #432]
	add w14, w14, w9
	add w14, w14, #1
	cmp x14, x13
	b.eq .LBB9_10
	bfi x12, x14, #32, #32
	dmb ish
	b .LBB9_2
.LBB9_10:
	tbnz w8, #0, .LBB9_17
	ldr x8, [x3]
	mov x21, x11
	mov x22, x1
	mov x23, x0
	cbnz x8, .LBB9_13
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
	b .LBB9_1
.LBB9_14:
	stp xzr, xzr, [x0]
.LBB9_15:
	ldp x20, x19, [sp, #96]
	ldr x25, [sp, #48]
	ldp x22, x21, [sp, #80]
	ldp x24, x23, [sp, #64]
	ldp x29, x30, [sp, #32]
	add sp, sp, #112
	ret
.LBB9_16:
	ldr x8, [x1, #416]
	str xzr, [x0]
	add x8, x8, x10, lsl #4
	stp x8, x13, [x0, #8]
	b .LBB9_15
	mov w8, #1
	str x8, [x0]
	b .LBB9_15
