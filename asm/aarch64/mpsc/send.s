mpsc_send:
	sub sp, sp, #128
	stp x29, x30, [sp, #96]
	stp x20, x19, [sp, #112]
	add x29, sp, #96
	ldr x9, [x0]
	mov w8, #1
	mov x20, sp
	stp x8, x1, [sp]
	add x1, x9, #128
	stp x1, xzr, [sp, #16]
	ldar x4, [x1]
	mov w8, w4
	cmp x8, x4, lsr #32
	b.eq .LBB12_4
	ldr x10, [x9, #560]
	ldr x11, [x9, #552]
	and x10, x10, x4
	sub x11, x11, #1
	cmp x10, x11
	b.hs .LBB12_4
	add x11, x4, #1
	mov x12, x4
	cas x12, x11, [x1]
	cmp x12, x4
	mov x4, x12
	b.ne .LBB12_4
	ldr x9, [x9, #544]
	add x9, x9, x10, lsl #4
	b .LBB12_8
.LBB12_4:
	ldr x2, [x2]
	sub x0, x29, #24
	add x3, x20, #24
	bl chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold
	ldur w8, [x29, #-24]
	tbz w8, #0, .LBB12_7
	mov w0, #2
	ldr x8, [sp, #24]
	cbz x8, .LBB12_13
	b .LBB12_15
	ldp x9, x8, [x29, #-16]
.LBB12_8:
	ldr w10, [sp]
	ldr x19, [sp, #8]
	str xzr, [sp]
	tbz w10, #0, .LBB12_16
	cbz x9, .LBB12_14
	ldr x10, [sp, #16]
	str x19, [x9], #8
	stlr x8, [x9]
	add x8, x10, #344
	ldar x1, [x8]
	cmp x1, #1
	b.hi .LBB12_12
	add x0, x10, #312
	bl spmc_waker::SpmcWaker<_,_>::wake_unsync_cold
.LBB12_12:
	mov x0, xzr
	ldr x8, [sp, #24]
	cbnz x8, .LBB12_15
	mov x1, x19
	ldp x20, x19, [sp, #112]
	ldp x29, x30, [sp, #96]
	add sp, sp, #128
	ret
	mov w0, #1
	ldr x8, [sp, #24]
	cbz x8, .LBB12_13
.LBB12_15:
	mov x8, sp
	mov x20, x0
	add x0, x8, #24
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	mov x0, x20
	mov x1, x19
	ldp x20, x19, [sp, #112]
	ldp x29, x30, [sp, #96]
	add sp, sp, #128
	ret
	bl <chenal::channel::SendFuture<T,Ch,B> as core::future::future::Future>::poll::polled_after_completion
	brk #0x1
	ldr x8, [sp, #24]
	mov x19, x0
	cbnz x8, .LBB12_20
.LBB12_19:
	mov x0, x19
	bl _Unwind_Resume
	add x0, x20, #24
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	b .LBB12_19
	bl core::panicking::panic_in_cleanup
