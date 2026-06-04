spsc_recv:
	stp x29, x30, [sp, #-32]!
	str x19, [sp, #16]
	mov x29, sp
	ldr x19, [x0]
	ldr x2, [x19, #256]
	mov w8, w2
	cmp x8, x2, lsr #32
	b.eq .LBB2_4
.LBB2_1:
	ldr x8, [x19, #544]
	ldr x9, [x19, #536]
	and x11, x2, #0xffffffff00000000
	and x10, x8, x2
	orr w8, w2, w8
	sub x9, x9, #1
	add w8, w8, #1
	cmp x10, x9
	ldr x9, [x19, #528]
	orr x8, x8, x11
	csinc x8, x8, x2, eq
	ldr x1, [x9, x10, lsl #3]
	str x8, [x19, #256]
	add x8, x19, #416
	dmb ish
	ldar x8, [x8]
	cmp x8, #1
	b.ls .LBB2_3
	mov x0, xzr
	ldr x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
.LBB2_3:
	add x0, x19, #384
	mov x19, x1
	mov x1, x8
	bl spmc_waker::SpmcWaker<_,_>::wake_unsync_cold
	mov x1, x19
	mov x0, xzr
	ldr x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
.LBB2_4:
	ldr x1, [x1]
	add x0, x19, #128
	bl chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold
	cmp x0, #2
	b.eq .LBB2_6
	cmp x0, #1
	b.ne .LBB2_7
.LBB2_6:
	ldr x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
.LBB2_7:
	mov x2, x1
	b .LBB2_1
