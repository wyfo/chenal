spsc_recv_blocking:
	push rbx
	sub rsp, 48
	mov rbx, qword ptr [rdi]
	mov rdx, qword ptr [rbx + 256]
	mov rax, rdx
	shr rax, 32
	cmp edx, eax
	je .LBB2_4
	mov qword ptr [rsp + 8], rdx
.LBB2_2:
	mov rdx, qword ptr [rsp + 8]
	mov rsi, qword ptr [rbx + 536]
	mov rax, rsi
	and rax, rdx
	mov rcx, qword ptr [rbx + 520]
	mov rdi, qword ptr [rbx + 528]
	dec rdi
	cmp rax, rdi
	jne .LBB2_6
	or esi, edx
	inc esi
	movabs rdi, -4294967296
	and rdi, rdx
	or rdi, rsi
	jmp .LBB2_7
.LBB2_6:
	inc rdx
	mov rdi, rdx
.LBB2_7:
	mov rdx, qword ptr [rcx + 8*rax]
	xchg qword ptr [rbx + 256], rdi
	mov rsi, qword ptr [rbx + 464]
	cmp rsi, 1
	jbe .LBB2_8
	xor eax, eax
	add rsp, 48
	pop rbx
	ret
.LBB2_4:
	lea rsi, [rbx + 128]
	mov dword ptr [rsp + 40], 1000000000
	mov rdi, rsp
	lea rcx, [rsp + 16]
	call chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold
	cmp byte ptr [rsp], 1
	jne .LBB2_2
	mov eax, 1
	add rsp, 48
	pop rbx
	ret
.LBB2_8:
	add rbx, 432
	mov rdi, rbx
	mov rbx, rdx
	call qword ptr [rip + spmc_waker::SpmcWaker<_,_>::wake_unsync_cold@GOTPCREL]
	mov rdx, rbx
	xor eax, eax
	add rsp, 48
	pop rbx
	ret
