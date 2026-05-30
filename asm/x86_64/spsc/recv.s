spsc_recv:
	push rbx
	mov rdi, qword ptr [rdi]
	mov rdx, qword ptr [rdi + 256]
	mov rax, rdx
	shr rax, 32
	cmp edx, eax
	je .LBB2_1
.LBB2_3:
	mov rsi, qword ptr [rdi + 536]
	mov rax, rsi
	and rax, rdx
	mov rcx, qword ptr [rdi + 520]
	mov r8, qword ptr [rdi + 528]
	dec r8
	cmp rax, r8
	jne .LBB2_5
	or esi, edx
	inc esi
	movabs r8, -4294967296
	and r8, rdx
	or r8, rsi
	jmp .LBB2_6
.LBB2_5:
	inc rdx
	mov r8, rdx
.LBB2_6:
	mov rdx, qword ptr [rcx + 8*rax]
	xchg qword ptr [rdi + 256], r8
	mov rsi, qword ptr [rdi + 464]
	cmp rsi, 1
	jbe .LBB2_8
	xor eax, eax
	pop rbx
	ret
.LBB2_8:
	add rdi, 432
	mov rbx, rdx
	call qword ptr [rip + spmc_waker::SpmcWaker<_,_>::wake_unsync_cold@GOTPCREL]
	mov rdx, rbx
	xor eax, eax
	pop rbx
	ret
.LBB2_1:
	mov rbx, rdi
	add rdi, 128
	mov rsi, qword ptr [rsi]
	call chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold
	cmp rax, 2
	jne .LBB2_9
	mov eax, 2
	pop rbx
	ret
.LBB2_9:
	cmp rax, 1
	jne .LBB2_10
	mov eax, 1
	pop rbx
	ret
.LBB2_10:
	mov rdi, rbx
	jmp .LBB2_3
