;(() => {
	var implementors = Object.fromEntries([
		[
			'tower_governor',
			[
				[
					'impl <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html" title="trait core::cmp::PartialEq">PartialEq</a> for <a class="struct" href="tower_governor/key_extractor/struct.GlobalKeyExtractor.html" title="struct tower_governor::key_extractor::GlobalKeyExtractor">GlobalKeyExtractor</a>',
				],
				[
					'impl <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html" title="trait core::cmp::PartialEq">PartialEq</a> for <a class="struct" href="tower_governor/key_extractor/struct.PeerIpKeyExtractor.html" title="struct tower_governor::key_extractor::PeerIpKeyExtractor">PeerIpKeyExtractor</a>',
				],
				[
					'impl <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html" title="trait core::cmp::PartialEq">PartialEq</a> for <a class="struct" href="tower_governor/key_extractor/struct.SmartIpKeyExtractor.html" title="struct tower_governor::key_extractor::SmartIpKeyExtractor">SmartIpKeyExtractor</a>',
				],
				[
					'impl&lt;K: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html" title="trait core::cmp::PartialEq">PartialEq</a> + <a class="trait" href="tower_governor/key_extractor/trait.KeyExtractor.html" title="trait tower_governor::key_extractor::KeyExtractor">KeyExtractor</a>, M: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html" title="trait core::cmp::PartialEq">PartialEq</a> + RateLimitingMiddleware&lt;QuantaInstant&gt;&gt; <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html" title="trait core::cmp::PartialEq">PartialEq</a> for <a class="struct" href="tower_governor/governor/struct.GovernorConfigBuilder.html" title="struct tower_governor::governor::GovernorConfigBuilder">GovernorConfigBuilder</a>&lt;K, M&gt;',
				],
			],
		],
	])
	if (window.register_implementors) {
		window.register_implementors(implementors)
	} else {
		window.pending_implementors = implementors
	}
})()
//{"start":57,"fragment_lengths":[1874]}
