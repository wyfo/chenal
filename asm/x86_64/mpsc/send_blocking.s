mpsc_send_blocking:
	push r14
	push rbx
	sub rsp, 56
	mov rdx, rsi
	mov rbx, qword ptr [rdi]
	mov rax, qword ptr [rbx + 128]
	mov ecx, eax
	mov rsi, rax
	shr rsi, 32
	cmp rcx, rsi
	je .LBB13_3
	mov rsi, qword ptr [rbx + 544]
	and rsi, rax
	mov rdi, qword ptr [rbx + 536]
	dec rdi
	cmp rsi, rdi
	jae .LBB13_3
	lea rdi, [rax + 1]
	lock cmpxchg	qword ptr [rbx + 128], rdi
	jne .LBB13_3
	shl rsi, 4
	add rsi, qword ptr [rbx + 528]
	mov qword ptr [rsp + 8], rsi
	mov qword ptr [rsp + 16], rcx
.LBB13_6:
	mov qword ptr [rsi], rdx
	mov qword ptr [rsi + 8], rcx
	mov rsi, qword ptr [rbx + 512]
	cmp rsi, 1
	jbe .LBB13_7
.LBB13_8:
	xor eax, eax
	add rsp, 56
	pop rbx
	pop r14
	ret
.LBB13_3:
	mov r14, rdx
	mov dword ptr [rsp + 48], 1000000000
	lea rsi, [rbx + 128]
	lea rdi, [rsp + 8]
	lea rcx, [rsp + 24]
	mov rdx, rax
	call chenal::channel::Chan<T,Ch,SP>::acquire_slot_blocking_cold
	mov rsi, qword ptr [rsp + 8]
	test rsi, rsi
	je .LBB13_4
	mov rcx, qword ptr [rsp + 16]
	mov rdx, r14
	jmp .LBB13_6
.LBB13_4:
	mov eax, 1
	mov rdx, r14
	add rsp, 56
	pop rbx
	pop r14
	ret
.LBB13_7:
	add rbx, 480
	mov rdi, rbx
	mov rbx, rdx
	call qword ptr [rip + spmc_waker::SpmcWaker<_,_>::wake_unsync_cold@GOTPCREL]
	mov rdx, rbx
	jmp .LBB13_8
