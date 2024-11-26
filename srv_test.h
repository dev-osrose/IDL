#pragma once

/* Generated with IDL v0.1.5 */


#include "packetfactory.h"

#ifndef JSON_USE_IMPLICIT_CONVERSIONS
#define JSON_USE_IMPLICIT_CONVERSIONS 0
#include "json.hpp"
#endif
#include <bitset>

namespace RoseCommon {
namespace Packet {

class SrvTest : public CRosePacket {
    public:
        static constexpr ePacketType PACKET_ID = ePacketType::PAKWC_TEST;
        SrvTest();
        SrvTest(CRoseReader reader);
        SrvTest(SrvTest&&) = default;
        SrvTest& operator=(SrvTest&&) = default;
        ~SrvTest() = default;
        
        static constexpr size_t size();
        
        
        enum aaa : uint8_t {
            ABC = 0,
            DEF = 1,
            GHI = 2,
        };
        
        struct bbb : public ISerialize {
            bbb();
            bbb(std::string);
            bbb(const bbb&) = default;
            bbb(bbb&&) = default;
            bbb& operator=(const bbb&) = default;
            bbb& operator=(bbb&&) = default;
            virtual ~bbb() = default;
            
            static constexpr size_t size();
            
            operator std::string() const { return bbb; }
            bool isValid() const { return is_valid; }
            
            virtual bool read(CRoseReader&) override;
            virtual bool write(CRoseBasePolicy&) const override;
            
            private:
                std::string bbb;
                bool is_valid;
        };
        
        
        SrvTest& set_a(const uint32_t);
        uint32_t get_a() const;
        SrvTest& set_b(const uint32_t);
        uint32_t get_b() const;
        SrvTest& set_c(const uint32_t);
        uint32_t get_c() const;
        SrvTest& set_d(const uint32_t);
        uint32_t get_d() const;
        SrvTest& set_e(const uint32_t);
        uint32_t get_e() const;
        SrvTest& set_f(const uint32_t);
        uint32_t get_f() const;
        SrvTest& set_g(const uint32_t);
        uint32_t get_g() const;
        SrvTest& set_h(const uint32_t);
        uint32_t get_h() const;
        SrvTest& set_pote(const Pote);
        Pote get_pote() const;
        SrvTest& set_pote2(const Pote2);
        Pote2 get_pote2() const;
        SrvTest& set_x(const Aaa);
        Aaa get_x() const;
        SrvTest& set_y(const Bbb);
        Bbb get_y() const;
        
        
        static SrvTest create(const uint32_t& a, const uint32_t& b, const uint32_t& c, const uint32_t& d, const uint32_t& e, const uint32_t& f, const uint32_t& g, const uint32_t& h, const Pote& pote, const Pote2& pote2, const Aaa& x, const Bbb& y);
        static SrvTest create(const uint8_t* buffer);
        static std::unique_ptr<SrvTest> allocate(const uint8_t* buffer);
    
    protected:
        virtual bool pack(CRoseBasePolicy&) const override;
    
    private:
        std::bitset<8> bitset1;
        uint32_t c;
        std::bitset<48> bitset2;
        Pote pote;
        Pote2 pote2;
        Aaa x;
        Bbb y;
};

void to_json(nlohmann::json& j, const SrvTest::aaa& data);
void to_json(nlohmann::json& j, const SrvTest::bbb& data);

void to_json(nlohmann::json& j, const SrvTest& data);

}
}
