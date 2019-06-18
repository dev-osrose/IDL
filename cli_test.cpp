#include "cli_test.h"

using namespace RoseCommon;
using namespace RoseCommon::Packet;

CliTest::B::B() : is_valid(false) {}

CliTest::B::B(int data) : b(data), is_valid(false) {
    is_valid = true;
}

bool CliTest::B::read(CRoseReader& reader) {
    if (!reader.get_int(b)) return false;
    is_valid = true;
    return true;
}

bool CliTest::B::write(CRoseBasePolicy& writer) const {
    return true;
}

constexpr size_t CliTest::B::size() {
    size_t size = 0;
    size += sizeof(int);
    return size;
}

CliTest::A::A() : is_valid(false) {}

CliTest::A::A(B data) : a(data), is_valid(false) {
    is_valid = true;
}

bool CliTest::A::read(CRoseReader& reader) {
    if (!reader.get_B(a)) return false;
    is_valid = true;
    return true;
}

bool CliTest::A::write(CRoseBasePolicy& writer) const {
    return true;
}

constexpr size_t CliTest::A::size() {
    size_t size = 0;
    size += sizeof(B);
    return size;
}



CliTest::CliTest() : CRosePacket(CliTest::PACKET_ID) {}

CliTest::CliTest(CRoseReader reader) : CRosePacket(reader) {
    for (size_t index = 0; index < 42; ++index) {
        if (!reader.get_int(test[index])) {
            return;
        }
    }
    if (!reader.get_iserialize(test2)) {
        return;
    }
}

void CliTest::set_test(const std::array<int, 42>& test) {
    reset_size();
    this->test = test;
}

void CliTest::set_test(const int& test, size_t index) {
    reset_size();
    this->test[index] = test;
}

const std::array<int, 42>& CliTest::get_test() const {
    return test;
}

const int& CliTest::get_test(size_t index) const {
    return test[index];
}

void CliTest::set_test2(const CliTest::A test2) {
    reset_size();
    this->test2 = test2;
}

CliTest::A CliTest::get_test2() const {
    return test2;
}

CliTest CliTest::create(const std::array<int, 42>& test, const CliTest::A& test2) {
    CliTest packet;
    packet.set_test(test);
    packet.set_test2(test2);
    return packet;
}

CliTest CliTest::create(const uint8_t* buffer) {
    CRoseReader reader(buffer, CRosePacket::size(buffer));
    return CliTest(reader);
}

std::unique_ptr<CliTest> CliTest::allocate(const uint8_t* buffer) {
    CRoseReader reader(buffer, CRosePacket::size(buffer));
    return std::make_unique<CliTest>(reader);
}

bool CliTest::pack(CRoseBasePolicy& writer) const {
    for (const auto& elem : test) {
        if (!writer.set_int(elem)) {
            return false;
        }
    }
    if (!writer.set_iserialize(test2)) {
        return false;
    }
    return true;
}

constexpr size_t CliTest::size() {
    size_t size = 0;
    size += sizeof(int) * 42; // test
    size += A::size(); // test2
    return size;
}

