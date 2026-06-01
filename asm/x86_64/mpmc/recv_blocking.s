mpmc_recv_blocking:
	sub rsp, 56
	mov rsi, qword ptr [rdi]
	mov rax, qword ptr [rsi + 256]
	mov rdx, qword ptr [rsi + 560]
	and rdx, rax
	mov rcx, qword ptr [rsi + 544]
	mov rdi, rdx
	shl rdi, 4
	mov r8, qword ptr [rcx + rdi + 8]
	cmp r8, rax
	jne .LBB12_7
	add rcx, rdi
	mov rcx, qword ptr [rcx]
	#MEMBARRIER
	mov rdi, qword ptr [rsi + 552]
	dec rdi
	cmp rdx, rdi
	jne .LBB12_3
	mov rdx, qword ptr [rsi + 560]
	or rdx, rax
	inc rdx
	lock cmpxchg	qword ptr [rsi + 256], rdx
	je .LBB12_5
.LBB12_7:
	mov dword ptr [rsp + 48], 1000000000
	sub rsi, -128
	lea rdi, [rsp + 8]
	lea rcx, [rsp + 24]
	mov rdx, rax
	call chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold
	cmp byte ptr [rsp + 8], 1
	jne .LBB12_6
	mov eax, 1
	add rsp, 56
	ret
.LBB12_3:
	lea rdx, [rax + 1]
	lock cmpxchg	qword ptr [rsi + 256], rdx
	jne .LBB12_7
.LBB12_5:
	mov qword ptr [rsp + 16], rcx
.LBB12_6:
	mov rdx, qword ptr [rsp + 16]
	xor eax, eax
	add rsp, 56
	ret
