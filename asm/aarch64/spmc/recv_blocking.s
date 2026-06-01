spmc_recv_blocking:
	sub sp, sp, #80
	stp x29, x30, [sp, #48]
	str x19, [sp, #64]
	add x29, sp, #48
	ldr x8, [x0]
	ldr x9, [x8, #256]
	mov w10, w9
	mov x2, x9
	cmp x10, x9, lsr #32
	b.eq .LBB12_3
	ldr x10, [x8, #560]
	ldr x11, [x8, #544]
	dmb ishld
	and x10, x10, x9
	ldr x19, [x11, x10, lsl #3]
	dmb ishld
	ldr x11, [x8, #552]
	sub x11, x11, #1
	cmp x10, x11
	b.ne .LBB12_5
	ldr w10, [x8, #560]
	and x11, x9, #0xffffffff00000000
	orr w10, w9, w10
	add w10, w10, #1
	orr x10, x10, x11
	add x11, x8, #256
	casal x2, x10, [x11]
	cmp x2, x9
	b.eq .LBB12_6
.LBB12_3:
	mov w9, #51712
	mov x0, sp
	add x1, x8, #128
	movk w9, #15258, lsl #16
	add x3, sp, #16
	str w9, [sp, #40]
	bl chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold
	ldrb w8, [sp]
	cmp w8, #1
	b.ne .LBB12_8
	mov w0, #1
	ldp x29, x30, [sp, #48]
	ldr x19, [sp, #64]
	add sp, sp, #80
	ret
.LBB12_5:
	add x10, x9, #1
	add x11, x8, #256
	casal x2, x10, [x11]
	cmp x2, x9
	b.ne .LBB12_3
.LBB12_6:
	add x9, x8, #416
	ldar x1, [x9]
	cmp x1, #1
	b.ls .LBB12_9
.LBB12_7:
	str x19, [sp, #8]
.LBB12_8:
	ldr x1, [sp, #8]
	mov x0, xzr
	ldp x29, x30, [sp, #48]
	ldr x19, [sp, #64]
	add sp, sp, #80
	ret
.LBB12_9:
	add x0, x8, #384
	bl spmc_waker::SpmcWaker<_,_>::wake_unsync_cold
	b .LBB12_7
